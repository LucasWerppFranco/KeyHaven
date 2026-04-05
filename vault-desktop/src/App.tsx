import { Routes, Route, Navigate } from "react-router-dom";
import { useEffect, useState } from "react";
import { vaultExists, isUnlocked } from "./api";
import SetupPage from "./pages/SetupPage";
import UnlockPage from "./pages/UnlockPage";
import VaultPage from "./pages/VaultPage";
import "./styles.css";

function App() {
  const [loading, setLoading] = useState(true);
  const [hasVault, setHasVault] = useState(false);
  const [unlocked, setUnlocked] = useState(false);

  useEffect(() => {
    checkStatus();
  }, []);

  async function checkStatus() {
    try {
      const exists = await vaultExists();
      setHasVault(exists);

      if (exists) {
        const isUnlockedResult = await isUnlocked();
        setUnlocked(isUnlockedResult);
      }
    } catch (e) {
      console.error("Failed to check vault status:", e);
    } finally {
      setLoading(false);
    }
  }

  if (loading) {
    return (
      <div className="loading-screen">
        <div className="spinner"></div>
        <p>Loading KeyHaven...</p>
      </div>
    );
  }

  return (
    <div className="app">
      <Routes>
        <Route
          path="/"
          element={
            !hasVault ? (
              <Navigate to="/setup" />
            ) : !unlocked ? (
              <Navigate to="/unlock" />
            ) : (
              <Navigate to="/vault" />
            )
          }
        />
        <Route
          path="/setup"
          element={<SetupPage onSetupComplete={() => {
              setHasVault(true);
              setUnlocked(true);
            }} />
          }
        />
        <Route
          path="/unlock"
          element={
            <UnlockPage
              onUnlock={() => {
                setUnlocked(true);
              }}
            />
          }
        />
        <Route
          path="/vault/*"
          element={
            <VaultPage
              onLock={() => {
                setUnlocked(false);
              }}
            />
          }
        />
      </Routes>
    </div>
  );
}

export default App;
