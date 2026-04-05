import { useState } from "react";
import { initVault } from "../api";

interface SetupPageProps {
  onSetupComplete: () => void;
}

export default function SetupPage({ onSetupComplete }: SetupPageProps) {
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");

    if (password.length < 12) {
      setError("Password must be at least 12 characters");
      return;
    }

    if (password !== confirmPassword) {
      setError("Passwords do not match");
      return;
    }

    setLoading(true);
    try {
      await initVault(password);
      onSetupComplete();
    } catch (e) {
      setError(e as string);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="setup-page">
      <div className="setup-card">
        <div className="logo">
          <svg viewBox="0 0 24 24" width="64" height="64">
            <path
              fill="currentColor"
              d="M12,17A2,2 0 0,0 14,15C14,13.89 13.1,13 12,13A2,2 0 0,0 10,15A2,2 0 0,0 12,17M12,1A7,7 0 0,1 19,8C19,10.86 17.37,13.32 15,14.5V17A2,2 0 0,1 13,19H11A2,2 0 0,1 9,17V14.5C6.63,13.32 5,10.86 5,8A7,7 0 0,1 12,1Z"
            />
          </svg>
        </div>
        <h1>Welcome to KeyHaven</h1>
        <p className="subtitle">Create your vault to securely store passwords</p>

        <form onSubmit={handleSubmit}>
          {error && <div className="error">{error}</div>}

          <div className="form-group">
            <label>Master Password</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="At least 12 characters"
              required
              minLength={12}
            />
          </div>

          <div className="form-group">
            <label>Confirm Password</label>
            <input
              type="password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              placeholder="Confirm your password"
              required
            />
          </div>

          <button type="submit" disabled={loading} className="btn-primary">
            {loading ? "Creating Vault..." : "Create Vault"}
          </button>
        </form>

        <p className="hint">
          Remember: Your master password cannot be recovered if forgotten.
        </p>
      </div>
    </div>
  );
}
