"use client";
import { useEffect } from "react";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  useEffect(() => {
    document.addEventListener("contextmenu", (event) => event.preventDefault());
    document.addEventListener("keydown", (event) => {
      if (
          (event.ctrlKey && event.shiftKey && event.key === "C") ||
          (event.ctrlKey && event.key === "U") ||
          (event.ctrlKey && event.shiftKey && event.key === "I")
      ) {
        event.preventDefault();
      }
    });
  }, []);

  return (
    <html lang="en" className="dark">
      <body className={inter.className}>{children}</body>
    </html>
  );
}
