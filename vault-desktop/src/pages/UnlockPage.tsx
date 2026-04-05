import { useState } from "react";
import { unlockVault } from "../api";

interface UnlockPageProps {
  onUnlock: () => void;
}

export default function UnlockPage({ onUnlock }: UnlockPageProps) {
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setLoading(true);

    try {
      await unlockVault(password);
      onUnlock();
    } catch (e) {
      setError("Invalid password");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="unlock-page">
      <div className="unlock-card">
        <div className="logo">
          <svg viewBox="0 0 24 24" width="64" height="64">
            <path
              fill="currentColor"
              d="M12,17A2,2 0 0,0 14,15C14,13.89 13.1,13 12,13A2,2 0 0,0 10,15A2,2 0 0,0 12,17M12,1A7,7 0 0,1 19,8C19,10.86 17.37,13.32 15,14.5V17A2,2 0 0,1 13,19H11A2,2 0 0,1 9,17V14.5C6.63,13.32 5,10.86 5,8A7,7 0 0,1 12,1Z"
            />
          </svg>
        </div>
        <h1>KeyHaven</h1>
        <p className="subtitle">Unlock your vault</p>

        <form onSubmit={handleSubmit}>
          {error && <div className="error">{error}</div>}

          <div className="form-group">
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Enter master password"
              required
              autoFocus
            />
          </div>

          <button type="submit" disabled={loading} className="btn-primary">
            {loading ? "Unlocking..." : "Unlock"}
          </button>
        </form>
      </div>
    </div>
  );
}
