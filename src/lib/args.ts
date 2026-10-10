export function splitArgs(text: string): string[] {
  const args: string[] = [];
  let current = "";
  let quoted = false;
  let started = false;

  for (const character of text) {
    if (character === '"') {
      quoted = !quoted;
      started = true;
    } else if (/\s/.test(character) && !quoted) {
      if (started) args.push(current);
      current = "";
      started = false;
    } else {
      current += character;
      started = true;
    }
  }
  if (started) args.push(current);
  return args;
}

export function joinArgs(args: string[]): string {
  return args.map((arg) => (/\s/.test(arg) ? `"${arg}"` : arg)).join(" ");
}
