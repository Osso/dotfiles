import os

from flake8.run import _initpep8, \
                       check_file, \
                       skip_file


def _get_files(repo):
    for filename in repo[None].files():
        filepath = os.path.join(repo.root, filename)
        if not skip_file(filepath) and filename.endswith('.py'):
            yield filepath


def hg_precommit_hook(ui, repo, **kwargs):
    # import IPython
    # IPython.embed()
    _initpep8()
    complexity = ui.config('flake8', 'complexity', default=-1)

    warnings = 0
    for file_ in _get_files(repo):
        warnings += check_file(file_, complexity)

    strict = ui.configbool('flake8', 'strict', default=True)

    if strict:
        return warnings

    return 0
