import type { Session } from "@/lib/tauri/journal";

export const WEEKS = 26;

/** Seconds of play a day needs to reach each brighter level of the calendar. */
const LEVEL_LIMITS = [1800, 5400, 10800];

export interface CalendarDay {
  key: string;
  date: Date;
  seconds: number;
  level: number;
  future: boolean;
}

export interface Records {
  weekSeconds: number;
  streak: number;
  longest: Session | null;
}

const pad = (value: number) => String(value).padStart(2, "0");

export const dayKey = (date: Date): string => `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;

const addDays = (date: Date, days: number): Date => {
  const next = new Date(date);
  next.setDate(next.getDate() + days);
  return next;
};

const startOfDay = (date: Date): Date => new Date(date.getFullYear(), date.getMonth(), date.getDate());

/** Monday of the week the date falls in. */
const startOfWeek = (date: Date): Date => addDays(startOfDay(date), -((date.getDay() + 6) % 7));

export function levelOf(seconds: number): number {
  if (seconds <= 0) return 0;
  return 1 + LEVEL_LIMITS.filter((limit) => seconds >= limit).length;
}

/** Play time per calendar day, a session counting towards the day it started on. */
export function secondsPerDay(sessions: Session[]): Map<string, number> {
  const perDay = new Map<string, number>();
  for (const session of sessions) {
    const key = dayKey(new Date(session.startedAt * 1000));
    perDay.set(key, (perDay.get(key) ?? 0) + session.durationSecs);
  }
  return perDay;
}

/** Columns are weeks, oldest first; each holds seven days from Monday. */
export function buildCalendar(perDay: Map<string, number>, today: Date): CalendarDay[][] {
  const first = addDays(startOfWeek(today), -(WEEKS - 1) * 7);
  const todayStart = startOfDay(today).getTime();
  return Array.from({ length: WEEKS }, (_, week) =>
    Array.from({ length: 7 }, (_, weekday) => {
      const date = addDays(first, week * 7 + weekday);
      const key = dayKey(date);
      const seconds = perDay.get(key) ?? 0;
      return { key, date, seconds, level: levelOf(seconds), future: date.getTime() > todayStart };
    }),
  );
}

/** Days in a row with play; today not having been played yet does not break it. */
function currentStreak(perDay: Map<string, number>, today: Date): number {
  let cursor = startOfDay(today);
  if (!perDay.has(dayKey(cursor))) cursor = addDays(cursor, -1);
  let streak = 0;
  while ((perDay.get(dayKey(cursor)) ?? 0) > 0) {
    streak += 1;
    cursor = addDays(cursor, -1);
  }
  return streak;
}

export function recordsOf(sessions: Session[], perDay: Map<string, number>, today: Date): Records {
  const monday = startOfWeek(today);
  let weekSeconds = 0;
  for (let day = monday; day.getTime() <= today.getTime(); day = addDays(day, 1)) weekSeconds += perDay.get(dayKey(day)) ?? 0;

  const longest = sessions.reduce<Session | null>((best, session) => (best && best.durationSecs >= session.durationSecs ? best : session), null);

  return { weekSeconds, streak: currentStreak(perDay, today), longest };
}

export function secondsPerInstance(sessions: Session[]): [string, number][] {
  const perInstance = new Map<string, number>();
  for (const session of sessions) perInstance.set(session.instanceId, (perInstance.get(session.instanceId) ?? 0) + session.durationSecs);
  return [...perInstance].sort((a, b) => b[1] - a[1]);
}
