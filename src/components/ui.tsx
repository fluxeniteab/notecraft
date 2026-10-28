import { useEffect, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import * as Tooltip from "@radix-ui/react-tooltip";
import * as Dialog from "@radix-ui/react-dialog";
import { AlertTriangle, Check, Loader2, X } from "lucide-react";
import { cn } from "../lib/cn";

export const TooltipProvider = Tooltip.Provider;

export function Tip({
  label,
  children,
  side = "bottom",
  shortcut,
}: {
  label: string;
  children: ReactNode;
  side?: "top" | "bottom" | "left" | "right";
  shortcut?: string;
}) {
  return (
    <Tooltip.Root>
      <Tooltip.Trigger asChild>{children}</Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Content
          side={side}
          sideOffset={6}
          className="nc-tooltip"
        >
          {label}
          {shortcut && <Kbd>{shortcut}</Kbd>}
          <Tooltip.Arrow className="fill-[var(--nc-border-strong)]" />
        </Tooltip.Content>
      </Tooltip.Portal>
    </Tooltip.Root>
  );
}

export function IconButton({
  label,
  onClick,
  children,
  active,
  danger,
  disabled,
  small,
  shortcut,
  className,
  side = "bottom",
}: {
  label: string;
  onClick?: (event: React.MouseEvent<HTMLButtonElement>) => void;
  children: ReactNode;
  active?: boolean;
  danger?: boolean;
  disabled?: boolean;
  small?: boolean;
  shortcut?: string;
  className?: string;
  side?: "top" | "bottom" | "left" | "right";
}) {
  return (
    <Tip label={label} side={side} shortcut={shortcut}>
      <button
        type="button"
        aria-label={label}
        aria-pressed={active}
        data-active={active || undefined}
        disabled={disabled}
        onClick={onClick}
        className={cn("nc-icon-btn", small && "nc-sm", danger && "nc-danger", className)}
      >
        {children}
      </button>
    </Tip>
  );
}

export function Kbd({ children }: { children: ReactNode }) {
  return <kbd className="nc-kbd">{children}</kbd>;
}

export function Spinner({ size = 14 }: { size?: number }) {
  return <Loader2 size={size} className="animate-spin" style={{ opacity: 0.75 }} />;
}

export function Switch({
  checked,
  onChange,
  label,
}: {
  checked: boolean;
  onChange: (value: boolean) => void;
  label: string;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      data-on={checked}
      className="nc-switch"
      onClick={() => onChange(!checked)}
    />
  );
}

export function Segmented<T extends string>({
  value,
  options,
  onChange,
  className,
}: {
  value: T;
  options: Array<{ value: T; label: ReactNode; title?: string }>;
  onChange: (value: T) => void;
  className?: string;
}) {
  return (
    <div className={cn("nc-seg", className)} role="tablist">
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          role="tab"
          aria-selected={value === option.value}
          title={option.title}
          data-active={value === option.value}
          className="nc-seg-btn"
          onClick={() => onChange(option.value)}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

export type MenuEntry =
  | {
      label: string;
      icon?: ReactNode;
      shortcut?: string;
      danger?: boolean;
      disabled?: boolean;
      onSelect: () => void;
    }
  | { separator: true }
  | { header: string };

export function ContextMenu({
  x,
  y,
  items,
  onClose,
}: {
  x: number;
  y: number;
  items: MenuEntry[];
  onClose: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState({ x, y });

  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    const pad = 8;
    setPos({
      x: Math.min(x, window.innerWidth - rect.width - pad),
      y: Math.min(y, window.innerHeight - rect.height - pad),
    });
  }, [x, y]);

  useEffect(() => {
    const onDown = (event: PointerEvent) => {
      if (!ref.current?.contains(event.target as Node)) onClose();
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("pointerdown", onDown);
    window.addEventListener("keydown", onKey);
    window.addEventListener("resize", onClose);
    window.addEventListener("blur", onClose);
    return () => {
      window.removeEventListener("pointerdown", onDown);
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("resize", onClose);
      window.removeEventListener("blur", onClose);
    };
  }, [onClose]);

  return createPortal(
    <div
      ref={ref}
      className="nc-menu"
      style={{ left: pos.x, top: pos.y }}
      role="menu"
      onContextMenu={(event) => event.preventDefault()}
    >
      {items.map((item, index) => {
        if ("separator" in item) return <div key={`sep-${index}`} className="nc-menu-sep" />;
        if ("header" in item) return <div key={`head-${index}`} className="nc-menu-label">{item.header}</div>;
        return (
          <button
            key={item.label}
            type="button"
            role="menuitem"
            disabled={item.disabled}
            data-danger={item.danger || undefined}
            className="nc-menu-item"
            onClick={() => {
              item.onSelect();
              onClose();
            }}
          >
            {item.icon}
            <span>{item.label}</span>
            {item.shortcut && <span className="nc-menu-shortcut"><Kbd>{item.shortcut}</Kbd></span>}
          </button>
        );
      })}
    </div>,
    document.body
  );
}

export function Modal({
  open,
  onOpenChange,
  children,
  className,
  labelledBy,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  children: ReactNode;
  className?: string;
  labelledBy?: string;
}) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="nc-overlay" />
        <Dialog.Content
          className={cn("nc-modal", className)}
          aria-labelledby={labelledBy}
          aria-describedby={undefined}
        >
          {children}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

export function DialogCloseButton({ onClose }: { onClose: () => void }) {
  return (
    <Dialog.Close asChild>
      <button type="button" aria-label="Close" className="nc-icon-btn" onClick={onClose}>
        <X size={15} />
      </button>
    </Dialog.Close>
  );
}

export function PromptDialog({
  open,
  title,
  label,
  initial = "",
  placeholder,
  confirmLabel = "Confirm",
  onClose,
  onSubmit,
}: {
  open: boolean;
  title: string;
  label: string;
  initial?: string;
  placeholder?: string;
  confirmLabel?: string;
  onClose: () => void;
  onSubmit: (value: string) => Promise<void> | void;
}) {
  const [value, setValue] = useState(initial);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setValue(initial);
      setError(null);
      setBusy(false);
    }
  }, [open, initial]);

  const submit = async () => {
    if (!value.trim() || busy) return;
    setBusy(true);
    setError(null);
    try {
      await onSubmit(value.trim());
      onClose();
    } catch (err) {
      setError(String(err).replace(/^Error:\s*/, ""));
      setBusy(false);
    }
  };

  return (
    <Modal open={open} onOpenChange={(next) => !next && onClose()} className="!w-[440px]" labelledBy="nc-prompt-title">
      <div className="nc-modal-head">
        <Dialog.Title id="nc-prompt-title" className="nc-modal-title">{title}</Dialog.Title>
        <div className="flex-1" />
        <DialogCloseButton onClose={onClose} />
      </div>
      <div className="nc-modal-body">
        <label className="mb-2 block text-[11px] font-medium text-fg-secondary">{label}</label>
        <input
          autoFocus
          value={value}
          placeholder={placeholder}
          onChange={(event) => setValue(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") void submit();
          }}
          className="nc-input"
        />
        {error && (
          <div className="mt-2 flex items-start gap-2 rounded-md border border-danger/20 bg-danger/5 px-2.5 py-2 text-[11px] text-danger">
            <AlertTriangle size={13} className="mt-0.5 shrink-0" />
            <span>{error}</span>
          </div>
        )}
      </div>
      <div className="nc-modal-foot">
        <button type="button" className="nc-btn nc-btn-outline" onClick={onClose}>Cancel</button>
        <button type="button" className="nc-btn nc-btn-primary" onClick={() => void submit()} disabled={!value.trim() || busy}>
          {busy ? <Spinner size={13} /> : <Check size={13} />}
          {confirmLabel}
        </button>
      </div>
    </Modal>
  );
}

export function ConfirmDialog({
  open,
  title,
  message,
  confirmLabel = "Delete",
  danger = true,
  onClose,
  onConfirm,
}: {
  open: boolean;
  title: string;
  message: ReactNode;
  confirmLabel?: string;
  danger?: boolean;
  onClose: () => void;
  onConfirm: () => Promise<void> | void;
}) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setBusy(false);
      setError(null);
    }
  }, [open]);

  const run = async () => {
    setBusy(true);
    setError(null);
    try {
      await onConfirm();
      onClose();
    } catch (err) {
      setError(String(err).replace(/^Error:\s*/, ""));
      setBusy(false);
    }
  };

  return (
    <Modal open={open} onOpenChange={(next) => !next && onClose()} className="!w-[440px]" labelledBy="nc-confirm-title">
      <div className="nc-modal-head">
        <div
          className={cn(
            "grid h-8 w-8 place-items-center rounded-lg",
            danger ? "bg-danger/10 text-danger" : "bg-accent-soft text-accent"
          )}
        >
          <AlertTriangle size={16} />
        </div>
        <Dialog.Title id="nc-confirm-title" className="nc-modal-title">{title}</Dialog.Title>
        <div className="flex-1" />
        <DialogCloseButton onClose={onClose} />
      </div>
      <div className="nc-modal-body">
        <div className="text-[12px] leading-relaxed text-fg-secondary">{message}</div>
        {error && <div className="mt-3 text-[11px] text-danger">{error}</div>}
      </div>
      <div className="nc-modal-foot">
        <button type="button" className="nc-btn nc-btn-outline" onClick={onClose}>Cancel</button>
        <button type="button" className={cn("nc-btn", danger ? "nc-btn-danger" : "nc-btn-primary")} onClick={() => void run()} disabled={busy}>
          {busy ? <Spinner size={13} /> : null}
          {confirmLabel}
        </button>
      </div>
    </Modal>
  );
}
