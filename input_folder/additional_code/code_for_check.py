import pandas as pd
import plotly.graph_objects as go
from plotly.subplots import make_subplots
from sklearn.metrics import accuracy_score, classification_report
import warnings
warnings.filterwarnings('ignore')

# ==========================================
# 1. Истинный датасет
# ==========================================
true_path = r'C:/hackatons/gazprom_hackaton/code_for_synthetic_generation/synthetic_datasets/bad_datasets/datasets_with_marks/synthetic_datasets_bad_contact/synthetic_dataset_bad_contact1.csv'
df_true = pd.read_csv(true_path, sep=';', encoding = 'cp1251')
df_true.columns = df_true.columns.str.strip()
df_true['TimeStamp'] = pd.to_datetime(df_true['TimeStamp'], errors='coerce')
df_true = df_true.sort_values('TimeStamp').reset_index(drop=True)
df_true['true_label'] = df_true['Anomaly_Class'].apply(lambda x: 1 if x == 0 else 0)

# ==========================================
# 2. Загрузка модельных файлов
# ==========================================
base_dir = r'C:/hackatons/gazprom_hackaton/code_for_synthetic_generation/Gazprom-hackathon-case3/model/output/'
model_files = [f'{base_dir}{i}.csv' for i in range(1, 18)]

def detect_separator(file_path):
    """Определяет разделитель по первой строке файла"""
    with open(file_path, 'r', encoding='utf-8') as f:
        first = f.readline()
    for sep in [';', '\t', ',']:
        if len(first.split(sep)) > 1:
            return sep
    return ';'

for i, file in enumerate(model_files, start=1):
    print(f"Обработка файла {i}...")
    try:
        sep = detect_separator(file)
        df_m = pd.read_csv(file, sep=sep, on_bad_lines='skip', engine='python')
    except Exception as e:
        print(f"  Ошибка чтения {file}: {e}. Пропускаем.")
        continue

    if df_m.empty:
        print(f"  Файл {i} пуст после пропуска битых строк. Пропускаем.")
        continue

    df_m.columns = df_m.columns.str.strip()

    # Определяем колонку с предсказаниями
    pred_col = None
    if '0 - Bad, 1 - Good' in df_m.columns:
        pred_col = '0 - Bad, 1 - Good'
    elif 'Anomaly_Class' in df_m.columns:
        pred_col = 'Anomaly_Class'
    elif 'prediction' in df_m.columns:
        pred_col = 'prediction'
    else:
        # Если нет стандартных колонок, берём первую числовую
        numeric_cols = df_m.select_dtypes(include=['number']).columns
        if len(numeric_cols) > 0:
            pred_col = numeric_cols[0]
            print(f"  Файл {i}: нет стандартных колонок, использую первую числовую: {pred_col}")
        else:
            print(f"  Файл {i}: не найдена колонка с предсказаниями. Пропускаем.")
            continue

    # Получаем предсказания в единой шкале: 1 = норма, 0 = аномалия
    if pred_col == 'Anomaly_Class':
        preds = df_m[pred_col].apply(lambda x: 1 if x == 0 else 0)
    else:
        preds = df_m[pred_col]

    preds = preds.fillna(1).astype(int)

    # Обрезаем или дополняем до длины df_true
    if len(preds) >= len(df_true):
        preds = preds.iloc[:len(df_true)]
    else:
        preds = pd.concat([preds, pd.Series([1] * (len(df_true) - len(preds)))], ignore_index=True)

    df_true[f'pred_{i}'] = preds.values
    print(f"  Файл {i}: колонка '{pred_col}', разделитель '{sep}', уникальные значения: {preds.unique()}")

# ==========================================
# 3. Формирование финального датасета
# ==========================================
pred_cols = [f'pred_{i}' for i in range(1, len(model_files)+1) if f'pred_{i}' in df_true.columns]
if not pred_cols:
    raise ValueError("Не удалось загрузить ни одного модельного предсказания!")

df_final = df_true.copy()
df_final['sum_pred'] = df_final[pred_cols].sum(axis=1)
df_final['final_model_label'] = (df_final['sum_pred'] == len(pred_cols)).astype(int)

# ==========================================
# 4. Оценка
# ==========================================
acc = accuracy_score(df_final['true_label'], df_final['final_model_label'])
print(f"\nAccuracy: {acc:.4f}")
print(classification_report(df_final['true_label'], df_final['final_model_label'], target_names=['Аномалия', 'Норма']))

# ==========================================
# 5. Визуализация (по индексу строк)
# ==========================================
sensor_cols = [col for col in df_true.columns if col not in ['TimeStamp', 'Anomaly_Class', 'true_label']
               and not col.startswith('pred_')]
sensors_to_plot = sensor_cols[:4]

df_final['idx'] = df_final.index

# Интервалы предсказанных аномалий
anom_intervals = df_final[df_final['final_model_label'] == 0].copy()
shaded_regions = []
if not anom_intervals.empty:
    anom_intervals['group'] = (anom_intervals['idx'].diff() != 1).cumsum()
    for _, group in anom_intervals.groupby('group'):
        shaded_regions.append((group['idx'].min(), group['idx'].max()))

fig = make_subplots(rows=len(sensors_to_plot), cols=1, shared_xaxes=True, vertical_spacing=0.05)

for i, s_name in enumerate(sensors_to_plot):
    # Сигнал датчика
    fig.add_trace(go.Scatter(x=df_final['idx'], y=df_final[s_name],
                             name=s_name, line=dict(color='lightgray'), showlegend=False), row=i+1, col=1)

    # Затенённые области (предсказанные аномалии)
    for (start, end) in shaded_regions:
        fig.add_vrect(x0=start, x1=end, fillcolor="orange", opacity=0.2,
                      line_width=0, row=i+1, col=1, layer="below",
                      showlegend=False if i>0 else True,
                      legendgroup="pred_anom", name="Предсказанная аномалия")

    # TP, FN, FP
    tp = df_final[(df_final['true_label'] == 0) & (df_final['final_model_label'] == 0)]
    fn = df_final[(df_final['true_label'] == 0) & (df_final['final_model_label'] == 1)]
    fp = df_final[(df_final['true_label'] == 1) & (df_final['final_model_label'] == 0)]

    fig.add_trace(go.Scatter(x=tp['idx'], y=tp[s_name], mode='markers',
                             name="Найденная аномалия (TP)",
                             marker=dict(color='green', size=7, symbol='circle')), row=i+1, col=1)
    fig.add_trace(go.Scatter(x=fn['idx'], y=fn[s_name], mode='markers',
                             name="Пропуск (FN)",
                             marker=dict(color='gold', size=8, symbol='triangle-up')), row=i+1, col=1)
    fig.add_trace(go.Scatter(x=fp['idx'], y=fp[s_name], mode='markers',
                             name="Ложная тревога (FP)",
                             marker=dict(color='red', size=8, symbol='x')), row=i+1, col=1)

fig.update_layout(height=300 * len(sensors_to_plot),
                  title_text="Детекция аномалий (по индексу строки): TP (зелёный), FN (жёлтый треугольник), FP (красный крест), оранжевая область – предсказание аномалии",
                  xaxis_title="Индекс строки",
                  showlegend=True)

# Убираем дублирование легенды для затенённых областей
for trace in fig.data:
    if trace.legendgroup == "pred_anom" and trace.name != "Предсказанная аномалия":
        trace.showlegend = False

fig.show()

# Дополнительная диагностика
print("\nДиагностика:")
print(f"Распределение true_label:\n{df_final['true_label'].value_counts()}")
print(f"Распределение final_model_label:\n{df_final['final_model_label'].value_counts()}")
tp_count = ((df_final['true_label'] == 0) & (df_final['final_model_label'] == 0)).sum()
fn_count = ((df_final['true_label'] == 0) & (df_final['final_model_label'] == 1)).sum()
fp_count = ((df_final['true_label'] == 1) & (df_final['final_model_label'] == 0)).sum()
print(f"TP: {tp_count}, FN: {fn_count}, FP: {fp_count}")