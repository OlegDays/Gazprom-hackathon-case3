import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

# ==========================================
# 1. Настройки параметров
# ==========================================
INPUT_FILE = '5.csv'
OUTPUT_FILE = 'synthetic_datasets/bad_datasets/datasets_with_marks/synthetic_datasets_partial_breakage/synthetic_datasets_partial_breakage5.csv'
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

        # --- Тип 1: частичный обрыв и RTD-эффекты+одиночные сверхбольшие выбросы
        for _ in range(np.random.randint(5, 8)):
            idx = np.random.randint(0, n - 250)
            length = np.random.randint(150, 251)

            # 1. Медленный дрейф (линейный тренд) по +-7,5% от амплитуды
            drift = np.linspace(0, mean_val * np.random.choice([0.075, -0.075]), length)
            signal[idx:idx + length] += drift

            # 2. Локальные зоны микро-шума
            micro_noise = np.random.normal(0, mean_val * 0.02, length)
            signal[idx:idx + length] += micro_noise

            # 3. Одиночные сверхкороткие выбросы (1-2 отсчета) к экстремальным значениям
            for _ in range(np.random.randint(1, 5)):  # 1-4 выброса внутри зоны
                spike_idx = idx + np.random.randint(0, length - 1)
                spike_len = np.random.randint(1, 3)
                # сверхкороткие выбросы по 30% от амплитуды
                signal[spike_idx:spike_idx + spike_len] += mean_val*np.random.choice([0.3, -0.3])

            labels[idx:idx + length] = np.maximum(labels[idx:idx + length], 3)

        # --- Тип 2: Частичный обрыв (средний) ---
        # 3-5 участков. Длительность: 1-2 сек (50-100 отсчетов)
        for _ in range(np.random.randint(2, 6)):
            idx = np.random.randint(0, n - 250)
            length = np.random.randint(150, 251)

            # 1. Медленный дрейф (линейный тренд) по +-10% от амплитуды
            drift = np.linspace(0, mean_val * np.random.choice([0.15, -0.15]), length)
            signal[idx:idx + length] += drift

            # 2. Локальные зоны микро-шума
            micro_noise = np.random.normal(0, mean_val * 0.05, length)
            signal[idx:idx + length] += micro_noise

            # 3. Одиночные сверхкороткие выбросы (1-2 отсчета) к экстремальным значениям
            for _ in range(np.random.randint(1, 5)):  # 1-4 выброса внутри зоны
                spike_idx = idx + np.random.randint(0, length - 1)
                spike_len = np.random.randint(1, 3)
                # сверхкороткие выбросы по 40% от амплитуды
                signal[spike_idx:spike_idx + spike_len] += mean_val * np.random.choice([0.4, -0.4])

            labels[idx:idx + length] = np.maximum(labels[idx:idx + length], 3)

        # --- Тип 3: Частичный обрыв и RTD-эффект (Дрейф) ---
        # 2-4 участка. Длительность: 3-5 сек (150-250 отсчетов)
        for _ in range(np.random.randint(1, 4)):
            idx = np.random.randint(0, n - 250)
            length = np.random.randint(150, 251)

            # 1. Медленный дрейф (линейный тренд)
            drift = np.linspace(0, mean_val * np.random.choice([0.2, -0.2]), length)
            signal[idx:idx + length] += drift

            # 2. Локальные зоны микро-шума
            micro_noise = np.random.normal(0, mean_val * 0.05, length)
            signal[idx:idx + length] += micro_noise

            # 3. Одиночные сверхкороткие выбросы (1-2 отсчета) к экстремальным значениям
            for _ in range(np.random.randint(1, 4)):  # 1-3 выброса внутри зоны
                spike_idx = idx + np.random.randint(0, length - 1)
                spike_len = np.random.randint(1, 3)
                # по +-60% от амплитуды
                signal[spike_idx:spike_idx + spike_len] += mean_val* np.random.choice([0.6, -0.6])

            labels[idx:idx + length] = np.maximum(labels[idx:idx + length], 3)

        df[col] = signal


    return labels

# Выбираем случайные колонки для внедрения дефектов (например, 5 датчиков)
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