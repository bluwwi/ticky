"use client";

import { useEffect, useState } from "react";

type KeyAction = "backspace" | "enter" | "space" | "shift" | "caps" | "tab";

type Key = {
  label: string;
  value?: string;
  grow?: number;
  action?: KeyAction;
  modifier?: boolean;
};

const rows: Key[][] = [
  [
    { label: "Esc", modifier: true },
    ...Array.from({ length: 12 }, (_, i) => ({
      label: `F${i + 1}`,
      modifier: true,
    })),
  ],
  [
    { label: "`", value: "`" },
    { label: "1", value: "1" },
    { label: "2", value: "2" },
    { label: "3", value: "3" },
    { label: "4", value: "4" },
    { label: "5", value: "5" },
    { label: "6", value: "6" },
    { label: "7", value: "7" },
    { label: "8", value: "8" },
    { label: "9", value: "9" },
    { label: "0", value: "0" },
    { label: "-", value: "-" },
    { label: "=", value: "=" },
    { label: "Backspace", action: "backspace", grow: 2, modifier: true },
  ],
  [
    { label: "Tab", action: "tab", grow: 1.5, modifier: true },
    { label: "Q", value: "q" },
    { label: "W", value: "w" },
    { label: "E", value: "e" },
    { label: "R", value: "r" },
    { label: "T", value: "t" },
    { label: "Y", value: "y" },
    { label: "U", value: "u" },
    { label: "I", value: "i" },
    { label: "O", value: "o" },
    { label: "P", value: "p" },
    { label: "[", value: "[" },
    { label: "]", value: "]" },
    { label: "\\", value: "\\", grow: 1.5 },
  ],
  [
    { label: "Caps", action: "caps", grow: 1.75, modifier: true },
    { label: "A", value: "a" },
    { label: "S", value: "s" },
    { label: "D", value: "d" },
    { label: "F", value: "f" },
    { label: "G", value: "g" },
    { label: "H", value: "h" },
    { label: "J", value: "j" },
    { label: "K", value: "k" },
    { label: "L", value: "l" },
    { label: ";", value: ";" },
    { label: "'", value: "'" },
    { label: "Enter", action: "enter", grow: 2.25, modifier: true },
  ],
  [
    { label: "Shift", action: "shift", grow: 2.25, modifier: true },
    { label: "Z", value: "z" },
    { label: "X", value: "x" },
    { label: "C", value: "c" },
    { label: "V", value: "v" },
    { label: "B", value: "b" },
    { label: "N", value: "n" },
    { label: "M", value: "m" },
    { label: ",", value: "," },
    { label: ".", value: "." },
    { label: "/", value: "/" },
    { label: "Shift", action: "shift", grow: 2.75, modifier: true },
  ],
  [
    { label: "Ctrl", grow: 1.5, modifier: true },
    { label: "Option", grow: 1.5, modifier: true },
    { label: "Space", action: "space", grow: 6, modifier: true },
    { label: "Option", grow: 1.5, modifier: true },
    { label: "Ctrl", grow: 1.5, modifier: true },
  ],
];

const keyBase =
  "flex h-14 shrink-0 items-start justify-start rounded-md border-2 border-zinc-500 bg-white p-1.25 text-left text-[13px] font-medium leading-none text-zinc-700 shadow-[0_1px_0_rgba(0,0,0,0.04)] transition-colors select-none hover:bg-zinc-50 active:translate-y-px active:bg-zinc-100";

export default function Keyboard() {
  const [shift, setShift] = useState(false);
  const [caps, setCaps] = useState(false);
  const [pressed, setPressed] = useState<string | null>(null);

  function press(key: Key) {
    if (key.action === "shift") {
      setShift((s) => !s);
      return;
    }
    if (key.action === "caps") {
      setCaps((c) => !c);
      return;
    }
    if (key.value !== undefined && shift) setShift(false);
  }

  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (e.ctrlKey || e.metaKey || e.altKey) return;

      if (
        e.key === "Backspace" ||
        e.key === "Enter" ||
        e.key === "Tab" ||
        e.key === " "
      ) {
        e.preventDefault();
      }

      setShift(e.shiftKey);
      setCaps(e.getModifierState("CapsLock"));
      setPressed(e.key);
    }

    function onKeyUp(e: KeyboardEvent) {
      setShift(e.shiftKey);
      setPressed(null);
    }

    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
    };
  }, []);

  return (
    <div className="flex flex-col p-10 gap-6">
      <div className="overflow-x-auto rounded-md ">
        <div className="flex w-max flex-col gap-1.5">
          {rows.map((row, i) => (
            <div key={i} className="flex gap-1.5">
              {row.map((key, j) => {
                const active =
                  (key.action === "shift" && shift) ||
                  (key.action === "caps" && caps);

                const isPressed =
                  pressed !== null &&
                  ((key.value !== undefined &&
                    key.value.toLowerCase() === pressed.toLowerCase()) ||
                    (key.action === "space" && pressed === " ") ||
                    (key.action === "enter" && pressed === "Enter") ||
                    (key.action === "backspace" && pressed === "Backspace") ||
                    (key.action === "tab" && pressed === "Tab") ||
                    (key.action === "shift" && pressed === "Shift") ||
                    (key.action === "caps" && pressed === "CapsLock"));

                return (
                  <button
                    key={`${i}-${j}`}
                    type="button"
                    onClick={() => press(key)}
                    style={{ width: `${(key.grow ?? 1) * 3.5}rem` }}
                    className={[
                      keyBase,
                      key.modifier ? "text-[10px] text-zinc-500" : "",
                      active || isPressed
                        ? "bg-zinc-100 text-zinc-900"
                        : "",
                    ].join(" ")}
                  >
                    {key.label}
                  </button>
                );
              })}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
