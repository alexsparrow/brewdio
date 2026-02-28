import { useEffect, useState } from "react";
import { Link } from "@tanstack/react-router";
import { getSyncStatus, onSyncStatusChange } from "@/lib/sync";

type SyncStatus = ReturnType<typeof getSyncStatus>;

const statusConfig: Record<
  SyncStatus | "offline",
  { label: string; dotClass: string }
> = {
  offline: { label: "Offline", dotClass: "bg-gray-400" },
  disconnected: { label: "Disconnected", dotClass: "bg-gray-400" },
  connecting: { label: "Connecting", dotClass: "bg-yellow-400" },
  connected: { label: "Connected", dotClass: "bg-green-400" },
};

export function SyncStatus() {
  const serverConfigured = !!localStorage.getItem("brewdio_server");
  const [status, setStatus] = useState<SyncStatus>(getSyncStatus());

  useEffect(() => {
    return onSyncStatusChange(setStatus);
  }, []);

  const displayStatus = serverConfigured ? status : "offline";
  const { label, dotClass } = statusConfig[displayStatus];

  return (
    <Link
      to="/settings"
      className="flex items-center gap-2 w-full px-2 py-1.5 text-sm rounded-md hover:bg-sidebar-accent transition-colors"
    >
      <span
        className={`inline-block h-2 w-2 rounded-full ${dotClass}`}
        aria-hidden
      />
      <span>{label}</span>
    </Link>
  );
}
