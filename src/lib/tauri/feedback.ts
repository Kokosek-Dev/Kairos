import { invoke } from "@tauri-apps/api/core";

export type FeedbackKind = "bug" | "idea" | "other";

export interface FeedbackReport {
  kind: FeedbackKind;
  message: string;
  contact: string | null;
  details: string | null;
}

export const feedbackAvailable = () => invoke<boolean>("feedback_available");
export const feedbackDetails = (instanceId: string | null) => invoke<string>("feedback_details", { instanceId });
export const sendFeedback = (report: FeedbackReport) => invoke<void>("send_feedback", { report });
