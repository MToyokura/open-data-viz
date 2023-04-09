import luigi
import processes.prefecture_map.luigi_module as prefecture_map_luigi_module


class AllTasks(luigi.WrapperTask):
    def requires(self):
        yield prefecture_map_luigi_module.ListFiles()
