import type { Session } from '@/types';
import { GroupCard } from './GroupCard';

export function GroupGrid({ groups, onSelect, activityLabel, locale }: {
  groups: Session[];
  locale?: string;
  onSelect: (id: string) => void;
  activityLabel: string;
}) {
  return (
    <div className="ui-grid">
      {groups.map((group) => <GroupCard key={group.id} group={group} onSelect={onSelect} activityLabel={activityLabel} locale={locale} />)}
    </div>
  );
}
