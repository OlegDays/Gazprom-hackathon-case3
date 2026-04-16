import pandas as pd
import plotly.graph_objects as go
from plotly.subplots import make_subplots
from sklearn.metrics import accuracy_score, classification_report

# ==========================================
# 1. Загрузка и подготовка истинного датасета
# ==========================================
true_path = r'C:/hackatons/gazprom_hackaton/code_for_synthetic_generation/synthetic_datasets/bad_datasets/datasets_with_marks/synthetic_datasets_bad_contact/synthetic_dataset_bad_contact1.csv'

df_true = pd.read_csv(true_path, sep=';')
df_true.columns = df_true.columns.str.strip()
# Укажите правильный формат времени, например '%Y-%m-%d %H:%M:%S.%f' или '%d.%m.%Y %H:%M:%S'
# Если формат неизвестен, можно использовать infer_datetime_format=True
df_true['TimeStamp'] = pd.to_datetime(df_true['TimeStamp'], infer_datetime_format=True)
df_true = df_true.sort_values('TimeStamp')

# Преобразуем классы аномалий: 0 (норма) -> 1; 1,2,3 (аномалия) -> 0
df_true['true_label'] = df_true['Anomaly_Class'].apply(lambda x: 1 if x == 0 else 0)

# ==========================================
# 2. Загрузка 20 модельных датасетов (по одному на датчик)
# ==========================================
# Предполагается, что файлы имеют имена synthetic_dataset_bad_contact1.csv ... synthetic_dataset_bad_contact20.csv
# и лежат в той же папке, что и истинный файл (или укажите свой путь)
# изменить пути к датасетам после получения датасетов
base_dir = r'C:/hackatons/gazprom_hackaton/code_for_synthetic_generation/synthetic_datasets/bad_datasets/datasets_with_marks/synthetic_datasets_bad_contact/'
model_files = [f'{base_dir}synthetic_dataset_bad_contact{i}.csv' for i in range(1, 2)]

model_dfs = []
for i, file in enumerate(model_files, start=1):
    df_m = pd.read_csv(file, sep=';')
    df_m.columns = df_m.columns.str.strip()
    df_m['TimeStamp'] = pd.to_datetime(df_m['TimeStamp'], infer_datetime_format=True)
    df_m = df_m.sort_values('TimeStamp')

    # Определяем колонку с предсказанием (обычно 'Anomaly_Class')
    pred_col = 'Anomaly_Class' if 'Anomaly_Class' in df_m.columns else 'prediction'
    # Оставляем только нужные колонки и переименовываем
    df_m = df_m[['TimeStamp', pred_col]].rename(columns={pred_col: f'pred_{i}'})
    model_dfs.append(df_m)

# Объединяем все модельные предсказания в один датафрейм по TimeStamp (outer merge)
df_models = model_dfs[0]
for df in model_dfs[1:]:
    df_models = pd.merge(df_models, df, on='TimeStamp', how='outer')
df_models = df_models.sort_values('TimeStamp').reset_index(drop=True)

# ==========================================
# 3. Объединение с истинными метками и вычисление итогового вердикта модели
# ==========================================
df_final = pd.merge(df_true, df_models, on='TimeStamp', how='left')

# Определяем колонки предсказаний динамически
pred_cols = [f'pred_{i}' for i in range(1, len(model_dfs) + 1)]
# Заполняем NaN значением 1 (норма) – по логике, если нет предсказания, считаем нормой
df_final[pred_cols] = df_final[pred_cols].fillna(1)

# Суммируем предсказания (1 – норма, 0 – аномалия)
df_final['sum_pred'] = df_final[pred_cols].sum(axis=1)

# Итоговый вердикт: 1 (норма) только если ВСЕ модели сказали 1 (норма), иначе 0 (аномалия)
THRESHOLD = len(model_dfs)  # 20
df_final['final_model_label'] = (df_final['sum_pred'] == THRESHOLD).astype(int)

# Сравнение
df_final['is_correct'] = df_final['true_label'] == df_final['final_model_label']

# ==========================================
# 4. Оценка качества
# ==========================================
acc = accuracy_score(df_final['true_label'], df_final['final_model_label'])
print(f"Accuracy: {acc:.4f}")
print(classification_report(df_final['true_label'], df_final['final_model_label'], target_names=['Аномалия', 'Норма']))

# ==========================================
# 5. Визуализация (первые 4 датчика)
# ==========================================
sensor_cols = [col for col in df_true.columns if col not in ['TimeStamp', 'Anomaly_Class', 'true_label']]
sensors_to_plot = sensor_cols[:4]

fig = make_subplots(rows=len(sensors_to_plot), cols=1, shared_xaxes=True, vertical_spacing=0.05)

for i, s_name in enumerate(sensors_to_plot):
    # Сигнал датчика
    fig.add_trace(go.Scatter(x=df_final['TimeStamp'], y=df_final[s_name],
                             name=f"{s_name}", line=dict(color='lightgray')), row=i + 1, col=1)
    # Реальные аномалии
    anom = df_final[df_final['true_label'] == 0]
    fig.add_trace(go.Scatter(x=anom['TimeStamp'], y=anom[s_name], mode='markers',
                             name="Реальная аномалия",
                             marker=dict(color='dodgerblue', size=6, symbol='circle')), row=i + 1, col=1)
    # Ошибки модели
    errors = df_final[df_final['is_correct'] == False]
    fig.add_trace(go.Scatter(x=errors['TimeStamp'], y=errors[s_name], mode='markers',
                             name="Ошибка ИИ",
                             marker=dict(color='red', size=8, symbol='x')), row=i + 1, col=1)

fig.update_layout(height=300 * len(sensors_to_plot), title_text="Сравнение аномалий и ошибок модели", showlegend=True)
fig.show()