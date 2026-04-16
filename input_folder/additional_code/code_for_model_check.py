import pandas as pd
import plotly.graph_objects as go
from plotly.subplots import make_subplots
from sklearn.metrics import classification_report
import os

# ==========================================
# ШАГ 1: Загрузка данных

true_path = r'C:/hackatons/gazprom_hackaton/code_for_synthetic_generation/Gazprom-hackathon-case3/input_folder/bad_datasets/synthetic_datasets_with_marks/datasets_with_marks/synthetic_datasets_bad_contact/synthetic_dataset_bad_contact1.csv'

# Способ 1: Самый надежный. Читаем только первую строку, чтобы узнать реальные имена колонок
temp_df = pd.read_csv(true_path, nrows=0, sep=None, engine='python')
print("Реальные колонки в файле:", temp_df.columns.tolist())

# Способ 2: Читаем файл с авто-определением разделителя (sep=None)
df_true = pd.read_csv(true_path, sep=';', engine='python')

# Убираем возможные пробелы из названий колонок
df_true.columns = df_true.columns.str.strip()

# Теперь конвертируем дату вручную, чтобы не ловить ошибки в read_csv
if 'TimeStamp' in df_true.columns:
    df_true['TimeStamp'] = pd.to_datetime(df_true['TimeStamp'])
else:
    # Если вдруг колонка называется иначе, выведем ошибку
    raise KeyError(f"Колонка 'TimeStamp' не найдена. Доступные колонки: {df_true.columns.tolist()}")

# Список файлов моделей (пока один для теста)
model_files = [
    r'C:\hackatons\gazprom_hackaton\code_for_synthetic_generation\Gazprom-hackathon-case3\input_folder\bad_datasets\synthetic_datasets_with_marks\datasets_with_marks\synthetic_datasets_bad_contact\synthetic_dataset_bad_contact1.csv'
]

model_datasets = []
for file in model_files:
    # Приводим TimeStamp к единому регистру при чтении
    df_m = pd.read_csv(file, parse_dates=['TimeStamp'], sep = ';')
    model_datasets.append(df_m)

# ==========================================
# ШАГ 2: Предобработка и Склейка
# ==========================================

# На скриншоте колонка Anomaly_Class. 0 - норма, остальное - аномалия.
# Превращаем в true_label: 1 (норма), 0 (аномалия)
df_true['true_label'] = df_true['Anomaly_Class'].apply(lambda x: 1 if x == 0 else 0)

# Берем за основу TimeStamp, true_label и все колонки датчиков из первого файла
# На скриншоте датчики имеют странные имена (кракозябры), выберем их по индексу или именам
df_final = df_true.copy()

# Добавляем предсказания каждой модели
for i, df_m in enumerate(model_datasets):
    # В моделях ищем колонку с предсказанием (например, 'Anomaly_Class' или 'prediction')
    # Для теста предположим, что модель тоже выдает 'Anomaly_Class'
    pred_col = 'Anomaly_Class' if 'Anomaly_Class' in df_m.columns else 'prediction'

    temp = df_m[['TimeStamp', pred_col]].rename(columns={pred_col: f'label_{i}'})

    # Приводим предсказание модели к формату 1 (норма) / 0 (аномалия)
    temp[f'label_{i}'] = temp[f'label_{i}'].apply(lambda x: 1 if x == 0 else 0)

    df_final = pd.merge(df_final, temp, on='TimeStamp', how='left')

# ==========================================
# ШАГ 3: Финальный вердикт
# ==========================================

# Считаем по тем колонкам label_i, которые реально добавились
label_cols = [c for c in df_final.columns if c.startswith('label_')]

df_final['sum_model_labels'] = df_final[label_cols].sum(axis=1)
# Если моделей меньше 20, меняем число здесь:
total_models = len(label_cols)
df_final['final_model_label'] = df_final['sum_model_labels'].apply(lambda x: 1 if x == total_models else 0)

df_final['is_correct'] = df_final['true_label'] == df_final['final_model_label']


# ==========================================
# ШАГ 4: Визуализация (Plotly)
# ==========================================

def render_final_report(df, sensor_names):
    fig = make_subplots(rows=len(sensor_names), cols=1, shared_xaxes=True, vertical_spacing=0.02)

    for i, s_name in enumerate(sensor_names):
        if s_name not in df.columns:
            continue

        # Основной сигнал
        fig.add_trace(go.Scatter(x=df['TimeStamp'], y=df[s_name], name=f"Датчик {i}",
                                 line=dict(color='rgba(150,150,150,0.5)')), row=i + 1, col=1)

        # Реальные аномалии (синие точки)
        anom = df[df['true_label'] == 0]
        fig.add_trace(go.Scatter(x=anom['TimeStamp'], y=anom[s_name], mode='markers',
                                 name="Реальная аномалия", marker=dict(color='blue', size=4)), row=i + 1, col=1)

        # Ошибки модели (красные крестики)
        errors = df[df['is_correct'] == False]
        fig.add_trace(go.Scatter(x=errors['TimeStamp'], y=errors[s_name], mode='markers',
                                 name="Ошибка ИИ", marker=dict(color='red', symbol='x', size=7)), row=i + 1, col=1)

    fig.update_layout(height=400 * len(sensor_names), template="plotly_white", showlegend=False)
    fig.show()


# Названия колонок датчиков с вашего скриншота (можно скопировать из df_true.columns)
# Для примера возьмем 3-ю и 4-ю колонки
sensors_to_plot = df_true.columns[2:]
render_final_report(df_final, sensors_to_plot)

print("--- Отчет по классификации ---")
print(classification_report(df_final['true_label'], df_final['final_model_label']))