import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Zap, Check, Trash2, EyeOff, Moon, Settings2, BellOff, CloudOff, Shield, RotateCcw, Gauge, MonitorDown } from 'lucide-react';

interface OptimizeResult {
  task: string;
  success: boolean;
  output: string;
}

const TASKS = [
  { id: 'clean_temp', label: '清理临时文件', desc: '删除系统临时目录中的缓存文件', icon: Trash2 },
  { id: 'clean_recycle', label: '清空回收站', desc: '永久删除回收站中的所有文件', icon: RotateCcw },
  { id: 'disable_hibernate', label: '禁用休眠', desc: '关闭休眠功能释放磁盘空间（约等于内存大小）', icon: Moon },
  { id: 'disable_visual_effects', label: '关闭视觉效果', desc: '调整为最佳性能模式，加快系统响应', icon: EyeOff },
  { id: 'optimize_services', label: '优化系统服务', desc: '禁用 SysMain/Superfetch、WSearch 等非必要服务', icon: Settings2 },
  { id: 'disable_telemetry', label: '禁用遥测', desc: '关闭 Windows 数据收集和诊断跟踪', icon: Shield },
  { id: 'disable_cortana', label: '关闭 Cortana', desc: '禁用 Cortana 语音助手及相关进程', icon: BellOff },
  { id: 'disable_onedrive', label: '关闭 OneDrive', desc: '停止并禁用 OneDrive 自动同步', icon: CloudOff },
  { id: 'disable_auto_maintenance', label: '禁用自动维护', desc: '关闭系统定时自动维护任务', icon: Gauge },
  { id: 'disable_startup_delay', label: '禁用启动延迟', desc: '移除启动时的系统延迟，加快开机速度', icon: MonitorDown },
];

export default function SystemOptimizeTool() {
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(false);
  const [results, setResults] = useState<OptimizeResult[]>([]);

  const toggleTask = (id: string) => {
    const next = new Set(selected);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    setSelected(next);
  };

  const selectAll = () => {
    setSelected(new Set(TASKS.map((t) => t.id)));
  };

  const clearAll = () => {
    setSelected(new Set());
  };

  const handleOptimize = async () => {
    if (selected.size === 0) return;
    setLoading(true);
    setResults([]);
    try {
      const data = await invoke<OptimizeResult[]>('run_optimize', {
        tasks: Array.from(selected),
      });
      setResults(data);
    } catch (e) {
      console.error(e);
    }
    setLoading(false);
  };

  return (
    <div className="max-w-3xl mx-auto space-y-5">
      <div className="flex items-center gap-3">
        <button onClick={selectAll} className="text-sm text-primary hover:underline">
          全选
        </button>
        <span className="text-text-secondary">|</span>
        <button onClick={clearAll} className="text-sm text-primary hover:underline">
          取消全选
        </button>
        <span className="text-text-secondary ml-auto text-sm">已选 {selected.size} 项</span>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        {TASKS.map((task) => {
          const Icon = task.icon;
          const isSelected = selected.has(task.id);
          return (
            <button
              key={task.id}
              onClick={() => toggleTask(task.id)}
              className={`flex items-center gap-3 p-4 rounded-xl border transition-all text-left ${
                isSelected
                  ? 'border-primary bg-primary-light'
                  : 'border-border bg-surface hover:border-primary/30'
              }`}
            >
              <div className={`w-10 h-10 rounded-lg flex items-center justify-center shrink-0 ${isSelected ? 'bg-primary text-white' : 'bg-background text-text-secondary'}`}>
                <Icon size={20} />
              </div>
              <div className="flex-1 min-w-0">
                <div className="font-medium text-text-primary text-sm">{task.label}</div>
                <div className="text-xs text-text-secondary mt-0.5">{task.desc}</div>
              </div>
              <div className={`w-5 h-5 rounded-full border-2 flex items-center justify-center shrink-0 ${isSelected ? 'border-primary bg-primary' : 'border-border'}`}>
                {isSelected && <Check size={12} className="text-white" />}
              </div>
            </button>
          );
        })}
      </div>

      <button
        onClick={handleOptimize}
        disabled={loading || selected.size === 0}
        className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-primary text-white text-base rounded-xl hover:bg-primary-dark transition-colors disabled:opacity-50"
      >
        <Zap size={18} />
        {loading ? '优化中...' : '一键优化'}
      </button>

      {results.length > 0 && (
        <div className="space-y-2">
          <h4 className="font-medium text-text-primary">执行结果</h4>
          {results.map((r, i) => (
            <div
              key={i}
              className={`p-3 rounded-lg text-sm ${r.success ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-700'}`}
            >
              <div className="font-medium">{TASKS.find((t) => t.id === r.task)?.label}</div>
              <div className="text-xs mt-1 opacity-80">{r.output}</div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
