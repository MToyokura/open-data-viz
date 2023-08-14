import luigi
from processes import luigi_module as processes_luigi_module


class AllTasks(luigi.WrapperTask):
    def requires(self):
        yield processes_luigi_module.AllTasks()
