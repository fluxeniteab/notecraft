import { useEffect, useMemo, useRef, useState } from "react";
import {
  ArrowLeft,
  Check,
  Eye,
  EyeOff,
  HardDrive,
  KeyRound,
  Loader2,
  Plus,
  ShieldCheck,
  UserRound,
} from "lucide-react";
import { api } from "../lib/tauri";
import type { ProfileSession, ProfileSummary } from "../lib/types";
import { isTauri } from "../lib/theme";

interface Props {
  onUnlocked: (session: ProfileSession) => void;
}

type Mode = "unlock" | "create";

/**
 * Local sign-in. Every profile is a separate encrypted vault on this machine;
 * the password is only ever used to derive the key, and is never stored.
 */
export default function LockScreen({ onUnlocked }: Props) {
  const [profiles, setProfiles] = useState<ProfileSummary[]>([]);
  const [mode, setMode] = useState<Mode>("unlock");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(true);
  const passwordRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    let alive = true;
    api
      .listProfiles()
      .then((list) => {
        if (!alive) return;
        setProfiles(list);
        if (list.length === 0) {
          setMode("create");
        } else {
          setUsername(list[0].username);
        }
      })
      .catch(() => {
        // Browser preview has no backend; the form still renders.
        setMode("create");
      })
      .finally(() => alive && setLoading(false));
    return () => {
      alive = false;
    };
  }, []);

  useEffect(() => {
    if (mode === "unlock" && username) passwordRef.current?.focus();
  }, [mode, username]);

  const selected = useMemo(
    () => profiles.find((profile) => profile.username === username) ?? null,
    [profiles, username]
  );

  const canSubmit = useMemo(() => {
    if (busy) return false;
    if (mode === "unlock") return password.length > 0;
    return username.trim().length >= 3 && password.length >= 8 && password === confirm;
  }, [busy, mode, password, confirm, username]);

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (!canSubmit) return;
    setBusy(true);
    setError(null);
    try {
      const summary =
        mode === "unlock"
          ? await api.unlockProfile(username, password)
          : await api.createProfile(username, password);
      setPassword("");
      setConfirm("");
      onUnlocked({ id: summary.id, username: summary.username });
    } catch (problem) {
      setError(String(problem));
    } finally {
      setBusy(false);
    }
  };

  const switchToCreate = () => {
    setMode("create");
    setUsername("");
    setPassword("");
    setConfirm("");
    setError(null);
  };

  const switchToUnlock = () => {
    setMode("unlock");
    setUsername(profiles[0]?.username ?? "");
    setPassword("");
    setConfirm("");
    setError(null);
  };

  return (
    <div className="nc-lock">
      <div className="nc-lock-shell">
        <aside className="nc-lock-aside">
          <div className="nc-lock-brand">
            <div className="nc-lock-mark">N</div>
            <div>
              <div className="nc-lock-brand-name">NoteCraft</div>
              <div className="nc-lock-brand-sub">Local-first notes</div>
            </div>
          </div>

          <div className="nc-lock-pitch">
            <h2>Your notes never leave this machine.</h2>
            <p>
              One encrypted vault per profile, opened with the password you choose. No
              server, no account, no telemetry.
            </p>
          </div>

          <ul className="nc-lock-points">
            <li>
              <ShieldCheck size={14} />
              <span>Argon2id key derivation, ChaCha20-Poly1305 at rest</span>
            </li>
            <li>
              <UserRound size={14} />
              <span>Separate usernames keep separate, isolated vaults</span>
            </li>
            <li>
              <HardDrive size={14} />
              <span>Stored inside this application's own data folder</span>
            </li>
          </ul>

          <div className="nc-lock-aside-foot">Nothing is uploaded. Ever.</div>
        </aside>

        <section className="nc-lock-panel">
          <div className="nc-lock-head">
            <div>
              <h1 className="nc-lock-title">
                {mode === "unlock" ? "Welcome back" : "Create your profile"}
              </h1>
              <p className="nc-lock-sub">
                {mode === "unlock"
                  ? "Unlock the vault with your password."
                  : "Pick a username and a password you will remember."}
              </p>
            </div>
          </div>

          {loading ? (
            <div className="nc-lock-loading">
              <Loader2 size={16} className="animate-spin" /> Checking profiles…
            </div>
          ) : (
            <form onSubmit={submit} className="nc-lock-form">
              {mode === "unlock" && profiles.length > 1 && (
                <div className="nc-lock-profiles" role="listbox" aria-label="Profiles">
                  {profiles.map((profile) => (
                    <button
                      key={profile.id}
                      type="button"
                      role="option"
                      aria-selected={profile.username === username}
                      className="nc-lock-profile"
                      data-active={profile.username === username || undefined}
                      onClick={() => {
                        setUsername(profile.username);
                        setPassword("");
                        setError(null);
                        passwordRef.current?.focus();
                      }}
                    >
                      <span className="nc-lock-avatar">
                        {profile.username.slice(0, 1).toUpperCase()}
                      </span>
                      <span className="truncate">{profile.username}</span>
                      {profile.username === username && <Check size={13} className="ml-auto" />}
                    </button>
                  ))}
                </div>
              )}

              {mode === "unlock" ? (
                <label className="nc-lock-field">
                  <span className="nc-lock-label">Profile</span>
                  <div className="nc-lock-input-wrap">
                    <UserRound size={14} />
                    <input
                      value={username}
                      onChange={(event) => setUsername(event.target.value)}
                      autoComplete="username"
                      spellCheck={false}
                      placeholder="Your name"
                    />
                  </div>
                </label>
              ) : (
                <label className="nc-lock-field">
                  <span className="nc-lock-label">Username</span>
                  <div className="nc-lock-input-wrap">
                    <UserRound size={14} />
                    <input
                      value={username}
                      onChange={(event) => setUsername(event.target.value)}
                      autoComplete="username"
                      spellCheck={false}
                      placeholder="ada"
                      autoFocus
                    />
                  </div>
                  <span className="nc-lock-hint">Each username keeps its own encrypted notes.</span>
                </label>
              )}

              <label className="nc-lock-field">
                <span className="nc-lock-label">Password</span>
                <div className="nc-lock-input-wrap">
                  <KeyRound size={14} />
                  <input
                    ref={passwordRef}
                    type={showPassword ? "text" : "password"}
                    value={password}
                    onChange={(event) => setPassword(event.target.value)}
                    autoComplete={mode === "unlock" ? "current-password" : "new-password"}
                    placeholder={mode === "unlock" ? "Your password" : "At least 8 characters"}
                  />
                  <button
                    type="button"
                    className="nc-lock-reveal"
                    aria-label={showPassword ? "Hide password" : "Show password"}
                    onClick={() => setShowPassword((value) => !value)}
                  >
                    {showPassword ? <EyeOff size={14} /> : <Eye size={14} />}
                  </button>
                </div>
              </label>

              {mode === "create" && (
                <label className="nc-lock-field">
                  <span className="nc-lock-label">Confirm password</span>
                  <div className="nc-lock-input-wrap">
                    <KeyRound size={14} />
                    <input
                      type={showPassword ? "text" : "password"}
                      value={confirm}
                      onChange={(event) => setConfirm(event.target.value)}
                      autoComplete="new-password"
                      placeholder="Repeat it"
                    />
                  </div>
                  {confirm.length > 0 && confirm !== password && (
                    <span className="nc-lock-error">Passwords do not match</span>
                  )}
                </label>
              )}

              {error && <div className="nc-lock-error nc-lock-error-block">{error}</div>}

              <button type="submit" className="nc-btn nc-btn-primary nc-lock-submit" disabled={!canSubmit}>
                {busy ? <Loader2 size={14} className="animate-spin" /> : <KeyRound size={14} />}
                {mode === "unlock" ? "Unlock" : "Create vault"}
              </button>

              <div className="nc-lock-switch">
                {mode === "unlock" ? (
                  <button type="button" onClick={switchToCreate}>
                    <Plus size={12} /> New profile
                  </button>
                ) : (
                  profiles.length > 0 && (
                    <button type="button" onClick={switchToUnlock}>
                      <ArrowLeft size={12} /> Back to unlock
                    </button>
                  )
                )}
              </div>
            </form>
          )}

          <p className="nc-lock-foot">
            {selected && mode === "unlock"
              ? selected.vault_path
              : isTauri()
                ? "Keep your password safe — there is no recovery if you lose it."
                : "Preview build — unlocking needs the desktop app."}
          </p>
        </section>
      </div>
    </div>
  );
}
