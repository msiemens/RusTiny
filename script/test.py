"""
The RusTiny test runner.

Test layout:

    tests/
        compile-fail/   # Tests that shouldn't compile
            [category]/
                [test].rs
        run-pass/       # Tests that *should* compile
            [category]/
                [test].rs
        ir/             # Test for the intermediate representation
            [test].rs   # Input
            [test].ir   # Expected IR

`compile-fail` tests can tell the test runner which error they expect by
using special comments:

    //! ERROR([line]:[col]): [ERROR MESSAGE]
    //! ERROR: [ERROR MESSAGE]

Tests can be skipped using:

    //! SKIP

To run only a subset of the test suites, pass them as an argument:

    python script/test.py asm,ir
"""

from collections import namedtuple
from glob import glob
from pathlib import Path
import os
import subprocess
import sys
import re

from termcolor import cprint, colored

import build


RUSTINY_DIR = Path(__file__).resolve().parents[1]
TEST_DIR = RUSTINY_DIR / 'tests'
COMPILER = RUSTINY_DIR / 'target' / 'debug' / 'rustiny'

if os.name == 'nt':
    import colorama
    colorama.init()

    COMPILER = COMPILER.with_suffix('.exe')


# --- HELPER CLASSES ----------------------------------------------------------

CompileResult = namedtuple('CompileResult', 'output exit_code')

FailedTest = namedtuple('FailedTest', 'path name unexpected missing output msg')


class CompilerError:
    def __init__(self, d: dict):
        self.error = d['error']
        self.line = int(d.get('line')) if 'line' in d else None
        self.col = int(d.get('col')) if 'col' in d else None

    def __repr__(self):
        if self.line and self.col:
            return 'Error in line {}:{}: {}'.format(self.line, self.col, self.error)
        else:
            return 'Error: {}'.format(self.error)

    def __eq__(self, other):
        assert isinstance(other, CompilerError)
        return (self.error == other.error
                and self.line == other.line
                and self.col == other.col)

    def __hash__(self):
        return hash((self.error, self.line, self.col))


class Session:
    def __init__(self):
        self.passed = 0
        self.failed = 0
        self.skipped = 0
        self.failures = []

    def start(self, name):
        print('Testing {} ...'.format(name), end='')

    def success(self):
        cprint('ok', 'green')

        self.passed += 1

    def failure(self, failure):
        cprint('failed', 'red')

        self.failures.append(failure)
        self.failed += 1

    def skip(self):
        cprint('skipped', 'yellow')

        self.skipped += 1

session = Session()


# --- Compile a file ----------------------------------------------------------

def compile_file(filename: Path, args=[]) -> CompileResult:
    proc = subprocess.Popen([str(COMPILER)] + args + [str(filename)],
                            stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT,
                            cwd=str(TEST_DIR))
    output = proc.communicate()[0].decode("utf-8")
    exit_code = proc.returncode

    return CompileResult(output, exit_code)


# --- Error and expectatino handling ------------------------------------------

def parse_errors(output: str):
    errors = []
    stderr = []

    for line in output.splitlines():
        match = re.match('Error in line (?P<line>\d+):(?P<col>\d+): ?'
                         '(?P<error>.*)', line)
        if match is not None:
            errors.append(CompilerError(match.groupdict()))
            continue

        match = re.match('Error: (?P<error>.*)', line)
        if match is not None:
            errors.append(CompilerError(match.groupdict()))
            continue

        stderr.append(line)

    return errors, stderr


def parse_expectations(filename: Path):
    expectations = []

    with filename.open(encoding='utf-8') as f:
        for line in f.readlines():
            match = re.match('.*//! ERROR\((?P<line>\d+):(?P<col>\d+)\): ?'
                             '(?P<error>.*)', line)
            if match is not None:
                expectations.append(CompilerError(match.groupdict()))
                continue

            match = re.match('.*//! ERROR: (?P<error>.*)', line)
            if match is not None:
                expectations.append(CompilerError(match.groupdict()))
                continue

    return expectations


# --- Compile a file ----------------------------------------------------------

def collect_categorized_tests(name):
    for cat in (TEST_DIR / name).iterdir():
        for test in sorted(list((TEST_DIR / cat).iterdir())):
            yield cat.name, test


def test_is_skip(filename):
    with filename.open(encoding='utf-8') as f:
        return '//! SKIP' in (line.strip() for line in f.readlines())


def tests_compiler():
    try:
        subprocess.check_call(['cargo', 'test'], cwd=str(RUSTINY_DIR))
    except subprocess.CalledProcessError:
        cprint('Compiler unit tests failed!', 'red')
        sys.exit(1)


def tests_compile_fail():
    for category, test in collect_categorized_tests('compile-fail'):
        test_name = test.name
        print('Testing {}/{} ... '.format(category, test_name), end='')

        if test_is_skip(test):
            session.skip()
            continue

        expectations = parse_expectations(test)
        cresult = compile_file(test)

        if cresult.exit_code == 0:
            session.failure(FailedTest(test, test_name, None, None, cresult.output,
                                       'compiling succeeded'))
        elif cresult.exit_code == 101:
            session.failure(FailedTest(test, test_name, None, None, cresult.output,
                                       'compiler panicked'))
        else:
            # Verify errors
            errors, stderr = parse_errors(cresult.output)
            unexpected_errors = set(errors) - set(expectations)
            missing_errors = set(expectations) - set(errors)

            if not unexpected_errors and not missing_errors:
                session.success()
            else:
                session.failure(FailedTest(test, test_name,
                                           unexpected_errors,
                                           missing_errors,
                                           '\n'.join(stderr),
                                           None))


def tests_run_pass():
    for category, test in collect_categorized_tests('run-pass'):
        test_name = test.parts[-1]
        print('Testing {}/{} ... '.format(category, test_name), end='')

        if test_is_skip(test):
            session.skip()
            continue

        cresult = compile_file(test)

        # Verify errors
        errors, stderr = parse_errors(cresult.output)

        if not errors and cresult.exit_code == 0:
            session.success()
        else:
            session.failure(FailedTest(test, test_name, errors, None,
                                       '\n'.join(stderr), None))


def tests_emit(target, ext, descr):
    tests = [name for name in sorted(list((TEST_DIR / target).iterdir()))
             if name.suffix == '.rs']

    for test in tests:
        test_name = test.parts[-1]
        print('Testing {} ... '.format(test_name), end='')

        if test_is_skip(test):
            session.skip()
            continue

        # Get generated IR
        cresult = compile_file(test, ['--target', target])

        if cresult.exit_code != 0:
            session.failure(FailedTest(test, test_name, None, None, cresult.output,
                                       'compiling failed'))
            continue

        generated = cresult.output.strip()

        # Get expceted IR
        with (TEST_DIR / target / (test.stem + ext)).open() as f:
            expected_ir = f.read().strip()

        if generated == expected_ir:
            session.success()
        else:
            output = '\n   {}\n{}\n\n   {}\n{}'.format(
                colored('Expected {}:'.format(descr), 'cyan'),
                expected_ir,
                colored('Generated {}:'.format(descr), 'cyan'),
                generated
            )
            session.failure(FailedTest(test, test_name, None, None,
                            None, output))


def print_results():
    print()

    pluralize_tests = lambda n: str(n) + (' tests' if n > 1 else ' test')

    if session.failed > 0:
        cprint('{} failed; '.format(pluralize_tests(session.failed)),
               'red', end='')
        if session.skipped:
            cprint('{} skipped; '.format(pluralize_tests(session.skipped)),
                   'yellow', end='')
        cprint('{} passed'.format(pluralize_tests(session.passed)), 'green')

        for failure in session.failures:
            print()
            print('--- Test {}: {}'.format(failure.path, failure.msg or ''))

            if failure.unexpected:
                print('Unexpected errors:')
                for unexpected in failure.unexpected:
                    print('   ' + str(unexpected))


            if failure.missing:
                print('Missing errors:')
                for missing in failure.missing:
                    print('   ' + str(missing))


            if failure.output:
                print('Compiler output:')
                print('   ' + '\n   '.join(failure.output.splitlines()))

    else:
        if session.skipped:
            cprint('{} skipped; '.format(pluralize_tests(session.skipped)),
                   'yellow', end='')

        cprint('{} passed'.format(pluralize_tests(session.passed)), 'green')


if __name__ == '__main__':
    os.environ['COLORED_OUTPUT'] = 'off'

    tests = {
        'internal': ('compiler unit tests', tests_compiler),
        'compile-fail': ('compile-fail tests', tests_compile_fail),
        'run-pass': ('run-pass tests', tests_run_pass),
        'ir': ('IR tests', lambda: tests_emit(target='ir', ext='.ir', descr='IR')),
        'asm': ('ASM tests', lambda: tests_emit(target='asm', ext='.s', descr='ASM'))
    }
    default_set = ['internal', 'compile-fail', 'run-pass', 'ir', 'asm']

    if len(sys.argv) == 2:
        suites = sys.argv[1].split(',')
    else:
        suites = default_set

    # Build the compiler in debug mode
    build.run('build', release=False)

    # Run specified set of test suites
    for suite in suites:
        title, func = tests[suite]

        cprint('Running {}...'.format(title), 'blue')
        func()

    print_results()

    if session.failed > 0:
        sys.exit(1)
