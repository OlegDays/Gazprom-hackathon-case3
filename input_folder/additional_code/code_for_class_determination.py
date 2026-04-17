import pandas as pd
#код используется для автоматического заполнения класса аномалии (в данном случае Anomaly_Class = 1)
data = pd.read_csv(r"C:\hackatons\gazprom_hackaton\code_for_synthetic_generation\5.csv", sep=';', encoding='cp1251')

data["Anomaly_Class"] = 1
data.to_csv(r"C:\hackatons\gazprom_hackaton\code_for_synthetic_generation\synthetic_datasets\good_datasets\5_with_marks.csv")
