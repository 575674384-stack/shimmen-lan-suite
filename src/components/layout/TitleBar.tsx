import { useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Minus, Square, X } from 'lucide-react';

export default function TitleBar() {
  const [isMaximized, setIsMaximized] = useState(false);
  const appWindow = getCurrentWindow();

  return (
    <div 
      data-tauri-drag-region 
      className="h-9 bg-surface border-b border-border flex items-center justify-between px-3 select-none shrink-0"
      style={{ WebkitAppRegion: 'drag' } as any}
    >
      <div className="flex items-center gap-2">
        <span className="text-sm font-semibold text-text-primary">水门内网协同</span>
      </div>
      <div className="flex items-center gap-0.5" style={{ WebkitAppRegion: 'no-drag' } as any}>
        <button onClick={() => appWindow.minimize()} className="w-7 h-7 flex items-center justify-center text-text-secondary hover:text-text-primary hover:bg-background rounded-md transition-colors">
          <Minus size={14} />
        </button>
        <button onClick={() => { appWindow.toggleMaximize(); setIsMaximized(!isMaximized); }} className="w-7 h-7 flex items-center justify-center text-text-secondary hover:text-text-primary hover:bg-background rounded-md transition-colors">
          <Square size={10} />
        </button>
        <button onClick={() => appWindow.hide()} className="w-7 h-7 flex items-center justify-center text-text-secondary hover:text-white hover:bg-red-500 rounded-md transition-colors">
          <X size={14} />
        </button>
      </div>
    </div>
  );
}
