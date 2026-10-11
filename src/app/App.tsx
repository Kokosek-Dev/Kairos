import { AnimatePresence, motion } from "framer-motion";
import { Toaster } from "sonner";
import { Backdrop } from "@/components/Backdrop";
import { GlowFollow } from "@/components/GlowFollow";
import { Mascot } from "@/components/Mascot";
import { IncognitoTheme } from "@/features/incognito/IncognitoTheme";
import { Route, Routes, useLocation } from "react-router-dom";
import { Sidebar } from "@/components/shell/Sidebar";
import { TitleBar } from "@/components/shell/TitleBar";
import { DiscoverPage } from "@/features/discover/DiscoverPage";
import { ProjectPage } from "@/features/discover/ProjectPage";
import { ContentPage } from "@/features/instances/ContentPage";
import { InstanceSettingsPage } from "@/features/instances/InstanceSettingsPage";
import { HomePage } from "@/features/home/HomePage";
import { SkinEditorPage } from "@/features/skins/editor/SkinEditorPage";
import { TexturesPage } from "@/features/textures/TexturesPage";
import { TexturePackPage } from "@/features/textures/TexturePackPage";
import { LibraryPage } from "@/features/library/LibraryPage";
import { SettingsPage } from "@/features/settings/SettingsPage";
import { OfflinePlayDialog } from "@/features/launch/OfflinePlayDialog";
import { SignInDialog } from "@/features/accounts/SignInDialog";
import { CommandPalette } from "@/features/palette/CommandPalette";
import { CrashDialog } from "@/features/launch/CrashDialog";
import { DetectiveDialog } from "@/features/detective/DetectiveDialog";
import { ConsoleDrawer } from "@/features/launch/ConsoleDrawer";
import { UpdateWatcher } from "@/features/updates/UpdateWatcher";
import { WelcomeDialog } from "@/features/welcome/WelcomeDialog";

export function App() {
  const location = useLocation();

  return (
    <div className="relative h-full">
      <Backdrop />
      <Mascot />
      <GlowFollow />
      <IncognitoTheme />
      <div className="relative z-10 flex h-full flex-col">
      <TitleBar />
      <div className="flex min-h-0 flex-1">
        <Sidebar />
        <main className="flex-1 overflow-y-auto p-8">
          <AnimatePresence mode="wait">
            <motion.div
              key={location.pathname}
              initial={{ opacity: 0, y: 14 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ duration: 0.2, ease: "easeOut" }}
            >
              <Routes location={location}>
                <Route path="/" element={<HomePage />} />
                <Route path="/library" element={<LibraryPage />} />
                <Route path="/skins/editor" element={<SkinEditorPage />} />
                <Route path="/textures" element={<TexturesPage />} />
                <Route path="/textures/:packId" element={<TexturePackPage />} />
                <Route path="/instances/:id" element={<InstanceSettingsPage />} />
                <Route path="/instances/:id/content" element={<ContentPage />} />
                <Route path="/discover" element={<DiscoverPage />} />
                <Route path="/discover/:projectId" element={<ProjectPage />} />
                <Route path="/settings" element={<SettingsPage />} />
              </Routes>
            </motion.div>
          </AnimatePresence>
        </main>
      </div>
      <ConsoleDrawer />
      <CrashDialog />
      <CommandPalette />
      </div>
      <DetectiveDialog />
      <SignInDialog />
      <OfflinePlayDialog />
      <Toaster position="bottom-right" theme="dark" toastOptions={{ style: { background: "var(--surface-raised)", color: "var(--foreground)", border: "1px solid var(--border)" } }} />
      <WelcomeDialog />
      <UpdateWatcher />
    </div>
  );
}
