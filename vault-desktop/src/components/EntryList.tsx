import { VaultEntry } from "../api";

interface EntryListProps {
  entries: VaultEntry[];
  loading: boolean;
  onRefresh: () => void;
}

export default function EntryList({ entries, loading }: EntryListProps) {
  if (loading) {
    return <div className="loading">Loading entries...</div>;
  }

  if (entries.length === 0) {
    return (
      <div className="empty-state">
        <p>No entries found.</p>
        <p>Add your first password to get started.</p>
      </div>
    );
  }

  return (
    <div className="entry-list">
      {entries.map((entry) => (
        <div key={entry.id} className="entry-card">
          <div className="entry-icon">
            {entry.title.charAt(0).toUpperCase()}
          </div>
          <div className="entry-info">
            <h3>{entry.title}</h3>
            <p>{entry.username || "No username"}</p>
            {entry.url && (
              <a href={entry.url} target="_blank" rel="noopener noreferrer">
                {entry.url}
              </a>
            )}
          </div>
          <div className="entry-actions">
            <button title="Copy password">
              <svg viewBox="0 0 24 24" width="18" height="18">
                <path
                  fill="currentColor"
                  d="M19,21H8V7H19M19,5H8A2,2 0 0,0 6,7V21A2,2 0 0,0 8,23H19A2,2 0 0,0 21,21V7A2,2 0 0,0 19,5M16,1H4A2,2 0 0,0 2,3V17H4V3H16V1Z"
                />
              </svg>
            </button>
            <button title="Edit">
              <svg viewBox="0 0 24 24" width="18" height="18">
                <path
                  fill="currentColor"
                  d="M20.71,7.04C21.1,6.65 21.1,6 20.71,5.63L18.37,3.29C18,2.9 17.35,2.9 16.96,3.29L15.12,5.12L18.87,8.87M3,17.25V21H6.75L17.81,9.93L14.06,6.18L3,17.25Z"
                />
              </svg>
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}
