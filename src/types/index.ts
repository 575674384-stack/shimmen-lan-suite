export interface User {
  id: string;
  username: string;
  ip: string;
  status: 'online' | 'offline';
  version: string;
}

export interface ChatMessage {
  id: string;
  sender_id: string;
  sender_name: string;
  type: 'text' | 'file' | 'image';
  content: string;
  timestamp: number;
}

export interface Task {
  id: string;
  title: string;
  project: string;
  deadline: string | null;
  contact: string;
  priority: 'low' | 'medium' | 'high';
  description: string;
  status: 'todo' | 'doing' | 'done';
  creator_id: string;
  assignee_id: string | null;
  is_team_visible: boolean;
  attached_files: string[];
  archived_to_folder_id?: string | null;
}

export interface PasswordEntry {
  id: string;
  name: string;
  account: string;
  password: string;
  note: string;
  created_by: string;
  updated_at: number;
}

export interface Announcement {
  id: string;
  title: string;
  content: string;
  is_pinned: boolean;
  created_by: string;
  updated_at: number;
}

export interface SharedFolder {
  id: string;
  owner_id: string;
  owner_name: string;
  local_path: string;
  name: string;
  sync_status: 'syncing' | 'paused' | 'error';
  ip?: string;
}

export interface TransferRecord {
  id: string;
  type: 'sent' | 'received';
  fileName: string;
  fileSize: string;
  progress: number;
  status: 'pending' | 'transferring' | 'completed' | 'failed';
}

export interface AvatarConfig {
  preset?: string;
  custom_path?: string;
}
