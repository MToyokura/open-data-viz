import typing


class RelevantInfo(typing.TypedDict):
    id: str
    statistics_name: str
    title: str
    survey_date: str
    updated_date: str
    item_name: str
    month: str


class TITLE_SPEC_TYPE(typing.TypedDict):
    TABLE_CATEGORY: str
    TABLE_NAME: str
    TABLE_SUB_CATEGORY1: str


class TableInfo(typing.TypedDict):
    # @id: str
    STAT_NAME: dict
    GOV_ORG: dict
    STATISTICS_NAME: str
    TITLE: dict
    CYCLE: str
    SURVEY_DATE: str
    OPEN_DATE: str
    SMALL_AREA: float
    COLLECT_AREA: str
    MAIN_CATEGORY: dict
    SUB_CATEGORY: dict
    OVERALL_TOTAL_NUMBER: float
    UPDATED_DATE: str
    STATISTICS_NAME_SPEC: dict
    DESCRIPTION: str
    TITLE_SPEC: TITLE_SPEC_TYPE


class PrefecturesTotal(typing.TypedDict):
    計: int
    北海道: int
    青森県: int
    岩手県: int
    宮城県: int
    秋田県: int
    山形県: int
    福島県: int
    茨城県: int
    栃木県: int
    群馬県: int
    埼玉県: int
    千葉県: int
    東京都: int
    神奈川県: int
    新潟県: int
    富山県: int
    石川県: int
    福井県: int
    山梨県: int
    長野県: int
    岐阜県: int
    静岡県: int
    愛知県: int
    三重県: int
    滋賀県: int
    京都府: int
    大阪府: int
    兵庫県: int
    奈良県: int
    和歌山県: int
    鳥取県: int
    島根県: int
    岡山県: int
    広島県: int
    山口県: int
    徳島県: int
    香川県: int
    愛媛県: int
    高知県: int
    福岡県: int
    佐賀県: int
    長崎県: int
    熊本県: int
    大分県: int
    宮崎県: int
    鹿児島県: int
    沖縄県: int


class DataWithAdditionalInfo(typing.TypedDict):
    year: str
    month: str
    item_name: str
    data: PrefecturesTotal
