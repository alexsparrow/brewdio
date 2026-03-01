import { Moon, Sun } from "lucide-react";
import { useEffect, useState } from "react";
import { SidebarMenuButton } from "@/components/ui/sidebar";

/**
 * Theme toggle component that switches between light and dark mode.
 * Persists theme preference to localStorage and updates the HTML class.
 */
export function ThemeToggle() {
  const [theme, setTheme] = useState<'light' | 'dark'>('dark');

  useEffect(() => {
    const stored = localStorage.getItem('theme');
    const systemPrefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;

    const initialTheme = stored === 'light' || stored === 'dark'
      ? stored
      : systemPrefersDark ? 'dark' : 'light';

    setTheme(initialTheme);
    document.documentElement.classList.toggle('dark', initialTheme === 'dark');
  }, []);

  const toggleTheme = () => {
    const newTheme = theme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
    localStorage.setItem('theme', newTheme);
    document.documentElement.classList.toggle('dark', newTheme === 'dark');
  };

  const label = theme === 'dark' ? 'Light Mode' : 'Dark Mode';

  return (
    <SidebarMenuButton tooltip={label} onClick={toggleTheme}>
      {theme === 'dark' ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
      <span>{label}</span>
    </SidebarMenuButton>
  );
}
