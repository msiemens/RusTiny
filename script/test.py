from collections import namedtuple
from pathlib import Path
import subprocess
import re
import colorama
from termcolor import cprint, colored

colorama.init()

RUSTINY_DIR = Path(__file__).resolve().parents[1]
TEST_DIR = RUSTINY_DIR / 'test'
COMPILER = RUSTINY_DIR / 'target' / 'rustiny'

CompileResult = namedtuple('CompileResult', ['output', 'exit_code'])


class CompilerError:
    def __init__(self, d: dict):
        self.error = d['error']
        self.line = int(d['line'])
        self.col = int(d['col'])

    def __repr__(self):
        return 'Error in {}:{}: {}'.format(self.line, self.col, self.error)

    def __eq__(self, other):
        assert isinstance(other, CompilerError)
        return self.error == other.error and self.line == other.line \
               and self.col == other.col

    def __hash__(self):
        return hash((self.error, self.line, self.col))


class Session:
    def __init__(self):
        self.passed = 0
        self.failed = 0


session = Session()


def compile_file(filename: Path) -> CompileResult:
    proc = subprocess.Popen([str(COMPILER), str(filename)],
                            stderr=subprocess.PIPE,
                            cwd=str(TEST_DIR))
    output = proc.communicate()[1].decode("utf-8")
    exit_code = proc.returncode

    return CompileResult(output, exit_code)


def parse_errors(output: str):
    errors = []

    for line in output.splitlines():
        match = re.match('Error in line (?P<line>\d+):(?P<col>\d+): '
                         '(?P<error>.*)', line)

        if match is not None:
            errors.append(CompilerError(match.groupdict()))

    return errors


def parse_expectations(filename: Path):
    expectations = []

    with filename.open() as f:
        for line in f.readlines():
            match = re.match('.* //! ERROR\((?P<line>\d+):(?P<col>\d+)\): '
                             '(?P<error>.*)', line)
            if match is not None:
                expectations.append(CompilerError(match.groupdict()))

    return expectations


def fail(msg=None):
    session.failed += 1

    cprint('failed ', 'red', end='')
    if msg:
        print('({})'.format(msg))
    else:
        print()


def success():
    session.passed += 1

    cprint('ok ', 'green')


def tests_compiler_unit():
    try:
        subprocess.check_call(['cargo', 'test'], cwd=str(RUSTINY_DIR))
    except subprocess.CalledProcessError:
        cprint('Compiler unit tests failed!', 'red')


def tests_compile_fail():
    # compile-fail
    for test in (TEST_DIR / 'compile-fail').iterdir():
        print('Testing {}... '.format(test.parts[-1]), end='')

        expectations = parse_expectations(test)
        cresult = compile_file(test)

        if cresult.exit_code == 0:
            fail('compiling succeeded')
        else:
            # Verify errors
            errors = parse_errors(cresult.output)
            unexpected_errors = set(errors) - set(expectations)
            missing_errors = set(expectations) - set(errors)

            if not unexpected_errors and not missing_errors:
                success()
            else:
                fail()
                if unexpected_errors:
                    print('Unexpected error(s):', unexpected_errors)
                if missing_errors:
                    print('Missing error(s):', missing_errors)


def print_results():
    print()

    if session.failed > 0:
        print('Passed: {}. Failed: {}'.format(
            colored(session.passed, 'green'),
            colored(session.failed, 'red')))
    else:
        print('All {} tests passed!'.format(colored(session.passed, 'green')))


def refresh_compiler():
    subprocess.check_call(['cargo', 'build'], cwd=str(RUSTINY_DIR))


if __name__ == '__main__':
    cprint('Running compiler unit tests...', 'blue')
    tests_compiler_unit()

    cprint('Running integration tests...', 'blue')
    refresh_compiler()
    tests_compile_fail()

    print_results()
