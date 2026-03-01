import { useEffect, useState } from "react";
import { Link } from "@tanstack/react-router";
import { getSyncStatus, onSyncStatusChange } from "@/lib/sync";
import { SidebarMenuButton } from "@/components/ui/sidebar";

type SyncStatusType = ReturnType<typeof getSyncStatus>;

const statusConfig: Record<
  SyncStatusType | "offline",
  { label: string; dotClass: string }
> = {
  offline: { label: "Offline", dotClass: "bg-gray-400" },
  disconnected: { label: "Disconnected", dotClass: "bg-gray-400" },
  connecting: { label: "Connecting", dotClass: "bg-yellow-400" },
  connected: { label: "Connected", dotClass: "bg-green-400" },
};

export function SyncStatus() {
  const serverConfigured = !!localStorage.getItem("brewdio_server");
  const [status, setStatus] = useState<SyncStatusType>(getSyncStatus());

  useEffect(() => {
    return onSyncStatusChange(setStatus);
  }, []);

  const displayStatus = serverConfigured ? status : "offline";
  const { label, dotClass } = statusConfig[displayStatus];

  return (
    <SidebarMenuButton tooltip={label} asChild>
      <Link to="/settings">
        <span
          className={`inline-block shrink-0 rounded-full ${dotClass} h-2 w-2 group-data-[collapsible=icon]:h-4 group-data-[collapsible=icon]:w-4`}
          aria-hidden
        />
        <span>{label}</span>
      </Link>
    </SidebarMenuButton>
  );
}
