import { useDroppable } from '@dnd-kit/core';
import type { Task } from '../../types';
import TaskCard from './TaskCard';

interface KanbanColumnProps {
  id: string;
  title: string;
  tasks: Task[];
  color: string;
  onTaskClick: (task: Task) => void;
}

export default function KanbanColumn({ id, title, tasks, color, onTaskClick }: KanbanColumnProps) {
  const { setNodeRef, isOver } = useDroppable({ id });

  return (
    <div className="flex-1 min-w-[280px] flex flex-col">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <div className={`w-3 h-3 rounded-full ${color}`} />
          <h3 className="font-bold text-base text-text-primary">{title}</h3>
        </div>
        <span className="text-sm text-text-secondary bg-background px-2 py-0.5 rounded-full">{tasks.length}</span>
      </div>
      <div
        ref={setNodeRef}
        className={`flex-1 bg-background rounded-xl p-3 space-y-3 transition-colors ${isOver ? 'bg-primary-light ring-2 ring-primary-light' : ''}`}
      >
        {tasks.map((task) => (
          <TaskCard key={task.id} task={task} onClick={() => onTaskClick(task)} />
        ))}
      </div>
    </div>
  );
}
