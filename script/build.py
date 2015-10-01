from pathlib import Path
import os
import shutil
import subprocess
import sys

from termcolor import cprint, colored

if os.name == 'nt':
    import colorama
    colorama.init()

RUSTINY_DIR = Path(__file__).resolve().parents[1]


def get_binary(name, release):
    if release is True:
        binary = RUSTINY_DIR / 'target' / 'debug' / name
    else:
        binary = RUSTINY_DIR / 'target' / 'release' / name

    if os.name == 'nt':
        binary = binary.with_suffix('.exe')

    return binary


def run(mode, release, args=None):
    cprint('Building instruction selection rules...', 'blue')
    build_rules(release)

    cprint('Building compiler...', 'blue')
    build_compiler(release)

    if mode == 'build':
        pass  # Nothing to do
    elif mode == 'run':
        compiler = RUSTINY_DIR / 'target' / 'debug' / 'rustiny'
        sys.exit(subprocess.call([str(compiler)] + args))
    elif mode == 'debug':
        compiler = RUSTINY_DIR / 'target' / 'debug' / 'rustiny'
        sys.exit(subprocess.call(['gdb', '--args', str(compiler), '--'] + args))
    else:
        cprint(('Unexpected mode:', mode))


def build_rules(release):
    rules_input = RUSTINY_DIR / 'src' / 'back' / 'instsel' / 'rules.ins.rs'
    rules_dummy = RUSTINY_DIR / 'src' / 'back' / 'instsel' / 'rules.dummy.rs'
    rules_dest = RUSTINY_DIR / 'src' / 'back' / 'instsel' / 'rules.rs'

    # Check if rules.dummy.rs is needed
    if not rules_dest.exists():
        copyfile(str(rules_dummy), str(rules_input))

    if rules_input.stat().st_mtime > rules_dest.stat().st_mtime:
        # Compile rules
        try:
            subprocess.check_call(['cargo', 'run', '--bin', 'rustiny-rulecomp',
                                   '--', '-o', 'src/back/instsel/rules.rs',
                                   'src/back/instsel/rules.ins.rs'],
                                  cwd=str(RUSTINY_DIR))
        except subprocess.CalledProcessError:
            cprint('Building rules failed', 'red')
            sys.exit(1)


def build_compiler(release):
    args = ['cargo', 'build']
    if release:
        args.append('--release')

    try:
        subprocess.check_call(args, cwd=str(RUSTINY_DIR))
    except subprocess.CalledProcessError:
        cprint('Building the compiler', 'red')
        sys.exit(1)


if __name__ == '__main__':
    sys.argv.pop(0)

    if not sys.argv:
        mode = 'build'
        release = False
        args = None
    else:
        if sys.argv[0] == '--release':
            sys.arv.pop(0)
            release = True
        else:
            release = False

        mode = sys.argv.pop(0)
        args = sys.argv

    run(mode, release, args)