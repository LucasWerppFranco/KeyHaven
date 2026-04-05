import { useState, useEffect } from "react";
import { Routes, Route, useNavigate } from "react-router-dom";
import { listEntries, lockVault, VaultEntry } from "../api";
import EntryList from "../components/EntryList";
import EntryForm from "../components/EntryForm";
import PasswordGenerator from "../components/PasswordGenerator";

interface VaultPageProps {
  onLock: () => void;
}

export default function VaultPage({ onLock }: VaultPageProps) {
  const navigate = useNavigate();
  const [entries, setEntries] = useState<VaultEntry[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<"entries" | "generator">("entries");

  useEffect(() => {
    loadEntries();
  }, [searchQuery]);

  async function loadEntries() {
    setLoading(true);
    try {
      const results = await listEntries(searchQuery || undefined);
      setEntries(results);
    } catch (e) {
      console.error("Failed to load entries:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleLock() {
    await lockVault();
    onLock();
  }

  return (
    <div className="vault-page">
      <header className="vault-header">
        <div className="logo-small">
          <svg viewBox="0 0 24 24" width="24" height="24">
            <path
              fill="currentColor"
              d="M12,17A2,2 0 0,0 14,15C14,13.89 13.1,13 12,13A2,2 0 0,0 10,15A2,2 0 0,0 12,17M12,1A7,7 0 0,1 19,8C19,10.86 17.37,13.32 15,14.5V17A2,2 0 0,1 13,19H11A2,2 0 0,1 9,17V14.5C6.63,13.32 5,10.86 5,8A7,7 0 0,1 12,1Z"
            />
          </svg>
          <span>KeyHaven</span>
        </div>

        <nav className="vault-nav">
          <button
            className={activeTab === "entries" ? "active" : ""}
            onClick={() => setActiveTab("entries")}
          >
            Entries
          </button>
          <button
            className={activeTab === "generator" ? "active" : ""}
            onClick={() => setActiveTab("generator")}
          >
            Generator
          </button>
        </nav>

        <button className="btn-lock" onClick={handleLock} title="Lock Vault">
          <svg viewBox="0 0 24 24" width="20" height="20">
            <path
              fill="currentColor"
              d="M12,17C10.89,17 10,16.1 10,15C10,13.89 10.89,13 12,13C13.1,13 14,13.89 14,15A2,2 0 0,1 12,17M12,9C13.1,9 14,9.89 14,11H10C10,9.89 10.89,9 12,9M5,21V9H7V7A5,5 0 0,1 12,2A5,5 0 0,1 17,7V9H19V21H5M17,7A3,3 0 0,0 14,4A3,3 0 0,0 11,7V9H17V7Z"
            />
          </svg>
          Lock
        </button>
      </header>

      <main className="vault-content">
        {activeTab === "entries" && (
          <>
            <div className="search-bar">
              <input
                type="text"
                placeholder="Search entries..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
              <button
                className="btn-add"
                onClick={() => navigate("/vault/new")}
              >
                + Add Entry
              </button>
            </div>

            <Routes>
              <Route
                path="/"
                element={
                  <EntryList
                    entries={entries}
                    loading={loading}
                    onRefresh={loadEntries}
                  />
                }
              />
              <Route
                path="/new"
                element={<EntryForm onSave={loadEntries} />}
              />
              <Route
                path="/edit/:id"
                element={<EntryForm onSave={loadEntries} />}
              />
            </Routes>
          </>
        )}

        {activeTab === "generator" && <PasswordGenerator />}
      </main>
    </div>
  );
}
