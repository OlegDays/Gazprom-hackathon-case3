import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

# ==========================================
# 1. Настройки параметров
# ==========================================
INPUT_FILE = '5.csv'
OUTPUT_FILE = 'synthetic_datasets/bad_datasets/datasets_with_marks/synthetic_datasets_other_contact_degradation/synthetic_datasets_contact_degradation5.csv'
N_ROWS = 10000
DT_MS = 20  # Частота дискретизации (мс)

# ==========================================
# 2. Подготовка базового набора данных
# ==========================================
# Читаем только первую строку исходного файла, чтобы забрать названия колонок
try:
    df_orig = pd.read_csv(INPUT_FILE, sep=';', nrows=1, encoding = 'cp1251')
    cols = [c for c in df_orig.columns if c != 'TimeStamp']
except FileNotFoundError:
    print(f"Файл {INPUT_FILE} не найден. Используем тестовые колонки.")
    cols = [f'Sensor_{i}' for i in range(1, 21)]

# Генерируем равномерную временную сетку (20 мс)
time_index = pd.date_range(start='2026-01-01 00:00:00', periods=N_ROWS, freq=f'{DT_MS}ms')
df_synth = pd.DataFrame({'TimeStamp': time_index.strftime('%H:%M:%S.%f').str[:-3]})
df_synth['Anomaly_Class'] = 0

# Генерируем базовые сигналы (псевдо-реальные данные: константа + случайное блуждание + шум)
np.random.seed(42)
base_means = {}
for col in cols:
    # Задаем случайное среднее значение для датчика от 50 до 3000
    mean_val = np.random.uniform(50, 3000)
    base_means[col] = mean_val
    # Базовый сигнал: среднее + медленный дрейф + базовый белый шум
    walk = np.cumsum(np.random.normal(0, mean_val * 0.0005, N_ROWS))
    noise = np.random.normal(0, mean_val * 0.005, N_ROWS)
    df_synth[col] = mean_val + walk + noise


# ==========================================
# 3. Математические модели дефектов
# ==========================================
def inject_anomalies(df, target_cols):
    n = len(df)
    labels = np.zeros(n, dtype=int)

    for col in target_cols:
        signal = df[col].values
        mean_val = base_means[col]

        # --- Тип 1: периодическая наводка (гармоническая помеха) ---
        # 5-15 участков. Возникает из-за ЭМ-полей от ЭД, частотных преобразователей, плохого экранирования кабеля.
        # Сигнал гуляет по "синусоиде". Выглядит как синусоида с небольшой амплитудой
        for _ in range(np.random.randint(5, 10)):
            duration_sec = np.random.uniform(2,5)
            idx = np.random.randint(0, n - 100)
            length = np.random.randint(50, 101)
            t_local = np.linspace(0,duration_sec, length)
            freq = np.random.uniform(1,10)
            interference = (mean_val*0.05)*np.sin(2*np.pi*freq*t_local)
            signal[idx:idx+length] += interference

        # --- Тип 2: Залипание сигнала. Возникает, когда АЦП не обновляет данные. Выглядит как замена участка массива одним и тем же числом.
        for _ in range(np.random.randint(3, 8)):
            idx = np.random.randint(0, n - 100)
            length = np.random.randint(50, 101)
            # Мгновенное смещение
            freeze_val = signal[idx-1]
            signal[idx:idx + length] = freeze_val
            #labels[idx:idx + length] = np.maximum(labels[idx:idx + length], 2)

        # --- Тип 3: Насыщение/срезание пиков.
        for _ in range(np.random.randint(2, 7)):
            idx = np.random.randint(0, n - 100)
            limit = mean_val * 1.5
            length = np.random.randint(50, 101)
            # Мгновенное смещение
            signal[idx:idx + length] = np.clip(signal[idx:idx+length],a_min = None, a_max = limit)

        # Тип 4: экспоненциальный дрейф (нагрев датчика). Возникает из-за перенагрева датчика
        for _ in range(np.random.randint(2, 7)):
            idx = np.random.randint(0, n - 100)
            length = np.random.randint(50, 101)
            t_local = np.linspace(0,3,length)
            drift = (mean_val * 0.2)*(1 - np.exp(-t_local))
            signal[idx:idx+length] += drift

        df[col] = signal


    return labels

# Выбираем случайные колонки для внедрения дефектов (например, 5 датчиков) (изменить при необходимости)
anom_cols = np.random.choice(cols, size=min(5, len(cols)), replace=False)
df_synth['Anomaly_Class'] = inject_anomalies(df_synth, anom_cols)

# ==========================================
# 4. Сохранение файла и визуализация
# ==========================================
# Сохраняем в CSV
df_synth.to_csv(OUTPUT_FILE, sep=';', index=False)
print(f"Синтетический датасет успешно сгенерирован и сохранен как '{OUTPUT_FILE}'")

# Строим график для визуальной проверки первого датчика с аномалиями (первые 2000 значений)
target_plot_col = anom_cols[0]

plt.figure(figsize=(16, 6))
plt.plot(df_synth[target_plot_col][:2000], label='Базовый сигнал + Дефекты', color='steelblue', linewidth=1.5)

# Подсвечиваем аномалии разными цветами
colors = {1: 'red', 2: 'orange', 3: 'purple'}
labels_names = {1: 'Шум (Тип 1)', 2: 'Ступенька (Тип 2)', 3: 'Дрейф (Тип 3)'}
subset_labels = df_synth['Anomaly_Class'][:2000].values

for cls in [1, 2, 3]:
    idx = np.where(subset_labels == cls)[0]
    if len(idx) > 0:
        plt.scatter(idx, df_synth[target_plot_col].iloc[idx], color=colors[cls], label=labels_names[cls], zorder=5,
                    s=15)

plt.title(f'Визуализация синтетических данных (Датчик: {target_plot_col}, первые 2000 отсчетов)')
plt.xlabel('Отсчеты (шаг 20 мс)')
plt.ylabel('Амплитуда сигнала')
plt.grid(True, linestyle='--', alpha=0.7)
plt.legend(loc='upper right')
plt.tight_layout()
plt.show()