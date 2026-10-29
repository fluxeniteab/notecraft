<script lang="ts">
  import { onMount } from "svelte";
  import { ArrowRight, Eye, EyeOff, KeyRound, Loader2, ShieldCheck, UserRound } from "lucide-svelte";
  import type { ProfileSummary } from "../lib/types";
  import { api } from "../lib/ipc";
  import { profile, pushToast } from "../lib/state";
  import { afterUnlock } from "../lib/actions";
  import { checkForUpdates } from "../lib/updater";

  let profiles: ProfileSummary[] = [];
  let mode: "unlock" | "create" = "unlock";
  let username = "";
  let password = "";
  let confirm = "";
  let showPassword = false;
  let busy = false;
  let error = "";
  let loading = true;
  let passwordInput: HTMLInputElement;

  $: selected = profiles.find((item) => item.username === username) ?? null;
  $: canSubmit =
    !busy &&
    (mode === "unlock"
      ? password.length > 0 && username.trim().length > 0
      : username.trim().length >= 3 && password.length >= 8 && password === confirm);

  onMount(() => {
    void (async () => {
      try {
        profiles = await api.listProfiles();
        if (profiles.length === 0) mode = "create";
        else username = profiles[0].username;
      } catch {
        mode = "create";
      } finally {
        loading = false;
        if (mode === "unlock") requestAnimationFrame(() => passwordInput?.focus());
      }
    })();
  });

  async function submit(): Promise<void> {
    if (!canSubmit) return;
    busy = true;
    error = "";
    try {
      const summary =
        mode === "unlock"
          ? await api.unlockProfile(username, password)
          : await api.createProfile(username.trim(), password);
      password = "";
      confirm = "";
      profile.set({ id: summary.id, username: summary.username });
      await afterUnlock();
      pushToast("success", mode === "unlock" ? "Profile unlocked" : "Profile created", summary.username);
      void checkForUpdates(true);
    } catch (problem) {
      error = String(problem);
    } finally {
      busy = false;
    }
  }

  function switchMode(next: "unlock" | "create"): void {
    mode = next;
    password = "";
    confirm = "";
    error = "";
    if (next === "create") username = "";
    else username = profiles[0]?.username ?? "";
  }
</script>

<div class="lock">
  <aside class="lock-side">
    <div class="lock-brand">
      <div class="brand-mark">N</div>
      <span>NoteCraft</span>
    </div>

    <div>
      <h1>Your notes, only on this machine.</h1>
      <p>
        Every profile is a local encrypted vault. Markdown, MDX and JSON documents stay in plain files
        you can read and back up — the password protects the vault, not your ability to leave.
      </p>
      <div style="margin-top:24px; display:flex; flex-direction:column; gap:9px">
        <div class="feature"><ShieldCheck size={13} /> Encrypted local profiles, no account required</div>
        <div class="feature"><KeyRound size={13} /> Keys are derived from your password and never stored</div>
        <div class="feature"><UserRound size={13} /> Separate vaults for separate parts of your life</div>
      </div>
    </div>

    <div class="lock-meta">
      <span class="status-dot"></span>
      Offline-first · NoteCraft 0.4.0
    </div>
  </aside>

  <main class="lock-main">
    <div class="lock-card">
      {#if loading}
        <div class="boot" style="height:auto; align-items:flex-start; gap:8px">
          <Loader2 size={16} class="spin" />
          <span>Looking for local profiles…</span>
        </div>
      {:else}
        <h2>{mode === "unlock" ? "Welcome back" : "Create a local profile"}</h2>
        <p class="sub">
          {mode === "unlock"
            ? "Unlock an encrypted vault on this machine."
            : "Choose a name and a password of at least 8 characters."}
        </p>

        <div class="lock-tabs" role="tablist">
          <button type="button" role="tab" class:on={mode === "unlock"} on:click={() => switchMode("unlock")}>Unlock</button>
          <button type="button" role="tab" class:on={mode === "create"} on:click={() => switchMode("create")}>Create</button>
        </div>

        {#if mode === "unlock" && profiles.length > 0}
          <div class="profile-list">
            {#each profiles as item}
              <button
                type="button"
                class="profile-row"
                class:on={item.username === username}
                on:click={() => {
                  username = item.username;
                  password = "";
                  requestAnimationFrame(() => passwordInput?.focus());
                }}
              >
                <span class="avatar">{item.username.slice(0, 1)}</span>
                <span class="who">
                  <strong>{item.username}</strong>
                  <small>{item.vault_path}</small>
                </span>
              </button>
            {/each}
          </div>
        {/if}

        <form class="lock-form" on:submit|preventDefault={submit}>
          {#if mode === "create" || !selected}
            <div class="field">
              <label for="lock-username">Profile name</label>
              <input
                id="lock-username"
                class="input"
                bind:value={username}
                autocomplete="username"
                placeholder="Your name"
                spellcheck="false"
              />
            </div>
          {/if}

          <div class="field">
            <label for="lock-password">Password</label>
            <div style="position:relative">
              <input
                id="lock-password"
                class="input"
                bind:this={passwordInput}
                bind:value={password}
                type={showPassword ? "text" : "password"}
                autocomplete={mode === "unlock" ? "current-password" : "new-password"}
                placeholder={mode === "unlock" ? "Vault password" : "At least 8 characters"}
                style="padding-right:34px"
              />
              <button
                type="button"
                class="icon-btn sm"
                style="position:absolute; right:3px; top:3px"
                aria-label={showPassword ? "Hide password" : "Show password"}
                on:click={() => (showPassword = !showPassword)}
              >
                {#if showPassword}<EyeOff size={13} />{:else}<Eye size={13} />{/if}
              </button>
            </div>
          </div>

          {#if mode === "create"}
            <div class="field">
              <label for="lock-confirm">Confirm password</label>
              <input
                id="lock-confirm"
                class="input"
                bind:value={confirm}
                type={showPassword ? "text" : "password"}
                autocomplete="new-password"
                placeholder="Repeat the password"
              />
            </div>
          {/if}

          {#if error}<div class="lock-error">{error}</div>{/if}

          <button type="submit" class="btn primary" style="height:34px; justify-content:center" disabled={!canSubmit}>
            {#if busy}<Loader2 size={14} class="spin" />{/if}
            {mode === "unlock" ? "Unlock vault" : "Create profile"}
            {#if !busy}<ArrowRight size={14} />{/if}
          </button>
        </form>

        <p class="lock-foot">
          There is no password reset. If you lose the password, the encrypted vault cannot be opened —
          keep a backup of the vault folder.
        </p>
      {/if}
    </div>
  </main>
</div>

<style>
  .feature {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--nc-fg-secondary);
    font-size: 12px;
  }
  .feature :global(svg) {
    color: var(--nc-fg-muted);
    flex: none;
  }
</style>
