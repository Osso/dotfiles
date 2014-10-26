import sublime_plugin


class PythonSnippetsPrintTraceback(sublime_plugin.TextCommand):
    def run(self, edit):
        for selection in self.view.sel():
            s = """import traceback; traceback.print_stack()"""
            self.view.replace(edit, selection, s)


class PythonSnippetsPdbTrace(sublime_plugin.TextCommand):
    def run(self, edit):
        for selection in self.view.sel():
            s = """import ipdb; ipdb.set_trace()"""
            self.view.replace(edit, selection, s)


class PythonSnippetsIpythonConsole(sublime_plugin.TextCommand):
    def run(self, edit):
        for selection in self.view.sel():
            s = """import IPython; IPython.embed()"""
            self.view.replace(edit, selection, s)


class PythonSnippetsSignedOff(sublime_plugin.TextCommand):
    def run(self, edit):
        for selection in self.view.sel():
            s = """Signed-off-by: Alessio Deiana <alessio.deiana@cern.ch>"""
            self.view.replace(edit, selection, s)

class PythonSnippetsTested(sublime_plugin.TextCommand):
    def run(self, edit):
        for selection in self.view.sel():
            s = """Tested-by: Alessio Deiana <alessio.deiana@cern.ch>"""
            self.view.replace(edit, selection, s)
