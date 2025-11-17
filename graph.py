import matplotlib.pyplot as plt
import numpy as np

# Data from the provided table
operations = ['CREATE', 'READ', 'UPDATE', 'DELETE']
python_latency = [28.9731, 9.4194, 53.4334, 48.2855]
rust_latency = [26.7005, 1.95, 1.2765, 6.4843]

# Set positions for bars
x = np.arange(len(operations))
width = 0.35  # width of the bars

fig, ax = plt.subplots(figsize=(8, 5))

# Plot bars
bar1 = ax.bar(x - width/2, python_latency, width, label='Python', color='skyblue')
bar2 = ax.bar(x + width/2, rust_latency, width, label='Rust', color='orange')

# Add labels and title
ax.set_xlabel('Operation')
ax.set_ylabel('Latency (ms)')
ax.set_title('Latency Comparison for 100 Records (Python vs Rust)')
ax.set_xticks(x)
ax.set_xticklabels(operations)
ax.legend()

# Add value labels on bars
for bars in [bar1, bar2]:
    for bar in bars:
        height = bar.get_height()
        ax.annotate(f'{height:.2f}',
                    xy=(bar.get_x() + bar.get_width() / 2, height),
                    xytext=(0, 3),  # offset
                    textcoords="offset points",
                    ha='center', va='bottom', fontsize=8)

plt.tight_layout()
plt.savefig('latency_comparison_100.png')
print("Bar chart saved as latency_comparison_100.png")