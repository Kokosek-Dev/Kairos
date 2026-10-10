import { AnimatePresence, motion } from "framer-motion";
import { Route, Routes, useLocation } from "react-router-dom";
import { Sidebar } from "@/components/shell/Sidebar";
import { TitleBar } from "@/components/shell/TitleBar";
import { DiscoverPage } from "@/features/discover/DiscoverPage";
import { ProjectPage } from "@/features/discover/ProjectPage";
import { ContentPage } from "@/features/instances/ContentPage";
import { InstanceSettingsPage } from "@/features/instances/InstanceSettingsPage";
import { LibraryPage } from "@/features/library/LibraryPage";
import { SettingsPage } from "@/features/settings/SettingsPage";
import { SignInDialog } from "@/features/accounts/SignInDialog";
import { ConsoleDrawer } from "@/features/launch/ConsoleDrawer";
import { VersionsPage } from "@/features/versions/VersionsPage";
import { WelcomeDialog } from "@/features/welcome/WelcomeDialog";

export function App() {
  const location = useLocation();

  return (
    <div className="flex h-full flex-col">
      <TitleBar />
      <div className="flex min-h-0 flex-1">
        <Sidebar />
        <main className="flex-1 overflow-y-auto p-8">
          <AnimatePresence mode="wait">
            <motion.div
              key={location.pathname}
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.15 }}
            >
              <Routes location={location}>
                <Route path="/" element={<LibraryPage />} />
                <Route path="/instances/:id" element={<InstanceSettingsPage />} />
                <Route path="/instances/:id/content" element={<ContentPage />} />
                <Route path="/versions" element={<VersionsPage />} />
                <Route path="/discover" element={<DiscoverPage />} />
                <Route path="/discover/:projectId" element={<ProjectPage />} />
                <Route path="/settings" element={<SettingsPage />} />
              </Routes>
            </motion.div>
          </AnimatePresence>
        </main>
      </div>
      <ConsoleDrawer />
      <SignInDialog />
      <WelcomeDialog />
    </div>
  );
}
