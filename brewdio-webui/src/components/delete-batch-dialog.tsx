import { useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import { deleteBatch } from "@/lib/actions/batches";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Trash2 } from "lucide-react";

interface DeleteBatchDialogProps {
  batchId: string;
  batchName: string;
  variant?: "default" | "destructive" | "outline" | "secondary" | "ghost" | "link";
  size?: "default" | "sm" | "lg" | "icon";
  showIcon?: boolean;
  redirectOnDelete?: boolean;
}

export function DeleteBatchDialog({
  batchId,
  batchName,
  variant = "destructive",
  size = "default",
  showIcon = true,
  redirectOnDelete = false,
}: DeleteBatchDialogProps) {
  const [open, setOpen] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const navigate = useNavigate();

  const handleDelete = async () => {
    setIsDeleting(true);
    try {
      await deleteBatch(batchId);
      setOpen(false);

      // Redirect to home page if requested (e.g., when deleting from batch page)
      if (redirectOnDelete) {
        navigate({ to: "/" });
      }
    } catch (error) {
      console.error("Failed to delete batch:", error);
    } finally {
      setIsDeleting(false);
    }
  };

  return (
    <AlertDialog open={open} onOpenChange={setOpen}>
      <AlertDialogTrigger asChild>
        <Button variant={variant} size={size}>
          {showIcon && <Trash2 className="h-4 w-4" />}
          {size !== "icon" && (showIcon ? <span className="ml-2">Delete</span> : "Delete")}
        </Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete Batch?</AlertDialogTitle>
          <AlertDialogDescription>
            Are you sure you want to delete <strong>{batchName}</strong>? This action cannot be undone.
            This will only delete the batch record, not the original recipe.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={isDeleting}>Cancel</AlertDialogCancel>
          <AlertDialogAction
            onClick={handleDelete}
            disabled={isDeleting}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
          >
            {isDeleting ? "Deleting..." : "Delete"}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
