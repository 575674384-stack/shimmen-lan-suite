import { useState } from 'react';
import {
  Globe, KeyRound, Monitor, Terminal, Zap, Search, FileEdit,
  ArrowLeft, Cpu, Printer, Network
} from 'lucide-react';
import DnsTool from './DnsTool';
import WinActivateTool from './WinActivateTool';
import EnvCheckTool from './EnvCheckTool';
import PwshTool from './PwshTool';
import SystemOptimizeTool from './SystemOptimizeTool';
import FileSearchTool from './FileSearchTool';
import BatchRenameTool from './BatchRenameTool';
import PrinterTool from './PrinterTool';
import NetworkInfoTool from './NetworkInfoTool';
import FileSearchNetwork from './FileSearchNetwork';

interface ToolCard {
  id: string;
  icon: React.ElementType;
  title: string;
  desc: string;
  color: string;
}

const tools: ToolCard[] = [
  { id: 'dns', icon: Globe, title: 'DNS优选', desc: '一键切换最快DNS服务器', color: 'from-sky-400 to-blue-500' },
  { id: 'activate', icon: KeyRound, title: 'Win10激活', desc: 'KMS一键激活Windows', color: 'from-emerald-400 to-green-500' },
  { id: 'env', icon: Monitor, title: '环境检测', desc: '查看本机硬件系统信息', color: 'from-violet-400 to-purple-500' },
  { id: 'pwsh', icon: Terminal, title: 'PS7 & UTF8', desc: '安装PowerShell7与UTF8', color: 'from-amber-400 to-orange-500' },
  { id: 'optimize', icon: Zap, title: '系统优化', desc: '一键清理与性能优化', color: 'from-rose-400 to-red-500' },
  { id: 'search', icon: Search, title: '文件查询', desc: '本机快速文件搜索', color: 'from-cyan-400 to-teal-500' },
  { id: 'rename', icon: FileEdit, title: '批量重命名', desc: '规则批量重命名文件', color: 'from-indigo-400 to-blue-500' },
  { id: 'printer', icon: Printer, title: '打印机管理', desc: '查看打印机与清空队列', color: 'from-pink-400 to-rose-500' },
  { id: 'network', icon: Network, title: '网络信息', desc: 'IP/MAC/网关/连通状态', color: 'from-cyan-400 to-blue-500' },
  { id: 'filesearch', icon: Search, title: '跨机搜文件', desc: '搜索内网所有电脑的文件', color: 'from-violet-400 to-purple-500' },
];

export default function ToolsPanel() {
  const [activeTool, setActiveTool] = useState<string | null>(null);

  const renderTool = () => {
    switch (activeTool) {
      case 'dns': return <DnsTool />;
      case 'activate': return <WinActivateTool />;
      case 'env': return <EnvCheckTool />;
      case 'pwsh': return <PwshTool />;
      case 'optimize': return <SystemOptimizeTool />;
      case 'search': return <FileSearchTool />;
      case 'rename': return <BatchRenameTool />;
      case 'printer': return <PrinterTool />;
      case 'network': return <NetworkInfoTool />;
      case 'filesearch': return <FileSearchNetwork />;
      default: return null;
    }
  };

  if (activeTool) {
    const tool = tools.find((t) => t.id === activeTool);
    return (
      <div className="h-full flex flex-col bg-background">
        <div className="h-14 bg-surface border-b border-border flex items-center px-5 gap-3 shrink-0">
          <button
            onClick={() => setActiveTool(null)}
            className="flex items-center gap-1.5 text-text-secondary hover:text-primary transition-colors"
          >
            <ArrowLeft size={18} />
            <span className="text-sm">返回工具箱</span>
          </button>
          <div className="w-px h-5 bg-border" />
          {tool && <tool.icon size={18} className="text-primary" />}
          <h3 className="font-bold text-base text-text-primary">{tool?.title}</h3>
        </div>
        <div className="flex-1 overflow-auto p-5">
          {renderTool()}
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col bg-background overflow-auto">
      <div className="p-6">
        <div className="flex items-center gap-3 mb-2">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-primary to-primary-dark flex items-center justify-center">
            <Cpu size={22} className="text-white" />
          </div>
          <div>
            <h2 className="font-bold text-xl text-text-primary">实用工具箱</h2>
            <p className="text-sm text-text-secondary">常用办公运维小工具集合</p>
          </div>
        </div>
      </div>

      <div className="px-6 pb-6 grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        {tools.map((tool) => {
          const Icon = tool.icon;
          return (
            <button
              key={tool.id}
              onClick={() => setActiveTool(tool.id)}
              className="bg-surface rounded-2xl p-5 border border-border hover:border-primary/30 hover:shadow-md transition-all text-left group"
            >
              <div className={`w-11 h-11 rounded-xl bg-gradient-to-br ${tool.color} flex items-center justify-center mb-3 group-hover:scale-105 transition-transform`}>
                <Icon size={22} className="text-white" />
              </div>
              <h4 className="font-semibold text-base text-text-primary mb-1">{tool.title}</h4>
              <p className="text-sm text-text-secondary">{tool.desc}</p>
            </button>
          );
        })}
      </div>
    </div>
  );
}
