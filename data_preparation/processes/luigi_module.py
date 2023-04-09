import luigi
import processes.monthly_vegetable_market_amount_by_prefecture.luigi_module as monthly_vegetable_market_luigi_module
import processes.prefecture_map.luigi_module as prefecture_map_luigi_module
import processes.workers_by_age_employment_type_education_sex.luigi_module as employment_type_luigi_module


class AllTasks(luigi.WrapperTask):
    def requires(self):
        yield prefecture_map_luigi_module.ListFiles()
        yield monthly_vegetable_market_luigi_module.ListFiles()
        yield employment_type_luigi_module.ListFiles()
