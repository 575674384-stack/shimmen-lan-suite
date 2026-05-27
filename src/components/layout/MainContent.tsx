import { useState } from 'react';
import Sidebar from './Sidebar';
import TitleBar from './TitleBar';
import UserList from '../user/UserList';
import ChatWindow from '../chat/ChatWindow';
import FolderBrowser from '../folder/FolderBrowser';
import PasswordVault from '../shared/PasswordVault';
import AnnouncementBoard from '../shared/AnnouncementBoard';
import KanbanBoard from '../board/KanbanBoard';
import TeamBoard from '../board/TeamBoard';
import ScreenSharePage from '../screen/ScreenSharePage';
import ToolsPanel from '../tools/ToolsPanel';
import SettingsModal from '../shared/SettingsModal';

export default function MainContent() {
  const [activeTab, setActiveTab] = useState('users');
  const [boardView, setBoardView] = useState<'personal' | 'team'>('personal');
  const [showSettings, setShowSettings] = useState(false);

  const renderContent = () => {
    switch (activeTab) {
      case 'users':
        return <UserList />;
      case 'chat':
        return <ChatWindow />;
      case 'folders':
        return <FolderBrowser />;
      case 'passwords':
        return <PasswordVault />;
      case 'announcements':
        return <AnnouncementBoard />;
      case 'board':
        return boardView === 'personal' ? <KanbanBoard /> : <TeamBoard />;
      case 'screen':
        return <ScreenSharePage />;
      case 'tools':
        return <ToolsPanel />;
      default:
        return <UserList />;
    }
  };

  return (
    <div className="h-screen w-screen flex flex-col bg-background overflow-hidden">
      <TitleBar />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar 
          activeTab={activeTab} 
          onTabChange={setActiveTab} 
          onOpenSettings={() => setShowSettings(true)}
        />
        <div className="flex-1 overflow-hidden">
          {activeTab === 'board' && (
            <div className="h-10 bg-surface border-b border-border px-4 flex items-center gap-4 shrink-0">
              <button 
                onClick={() => setBoardView('personal')} 
                className={`text-sm font-medium transition-colors ${boardView === 'personal' ? 'text-primary' : 'text-text-secondary hover:text-text-primary'}`}
              >
                个人看板
              </button>
              <button 
                onClick={() => setBoardView('team')} 
                className={`text-sm font-medium transition-colors ${boardView === 'team' ? 'text-primary' : 'text-text-secondary hover:text-text-primary'}`}
              >
                团队看板
              </button>
            </div>
          )}
          <div className={activeTab === 'board' ? 'h-[calc(100%-40px)]' : 'h-full'}>
            {renderContent()}
          </div>
        </div>
      </div>
      <SettingsModal isOpen={showSettings} onClose={() => setShowSettings(false)} />
    </div>
  );
}
