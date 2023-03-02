export type ResponseJson = {
  "id": string;
  "年齢階級(詳細集計）": string;
  "雇用形態": string;
  "教育": string;
  "性別": string;
  "data": { [key: string]: string | null };
};

export type PlotData = ResponseJson & { index: number };
