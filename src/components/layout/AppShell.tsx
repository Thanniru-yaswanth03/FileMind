import React, { useEffect } from 'react';
import { useApp } from '@/context/AppContext';
import { Sidebar } from './Sidebar';
import { TopToolbar } from './TopToolbar';
import { PropertyInspector } from './PropertyInspector';
import { StatusBar } from './StatusBar';
import { CommandPalette } from './CommandPalette';
import { NotificationCenter } from './NotificationCenter';
import { ContextMenu } from '../common/ContextMenu';
import { Toast } from '../common/Toast';

// Screens
import { OverviewScreen } from '@/screens/OverviewScreen';
import { StorageExplorerScreen } from '@/screens/StorageExplorerScreen';
import { LargeFilesScreen } from '@/screens/LargeFilesScreen';
import { DuplicateFinderScreen } from '@/screens/DuplicateFinderScreen';
import { CleanupPlannerScreen } from '@/screens/CleanupPlannerScreen';
import { SafetyCenterScreen } from '@/screens/SafetyCenterScreen';
import { DeveloperCleanupScreen } from '@/screens/DeveloperCleanupScreen';
import { AutomationsScreen } from '@/screens/AutomationsScreen';
import { JarvisScreen } from '@/screens/JarvisScreen';
import { ActivityRecoveryScreen } from '@/screens/ActivityRecoveryScreen';
import { SettingsScreen } from '@/screens/SettingsScreen';

export const AppShell: React.FC = () => {
  const { activeScreen, toggleSidebar } = useApp();

  // Keyboard navigation shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+B: Toggle Sidebar
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'b') {
        e.preventDefault();
        toggleSidebar();
      }
      // Ctrl+F: Focus search input
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'f') {
        e.preventDefault();
        const searchInput = document.querySelector<HTMLInputElement>('header input[type="text"]');
        searchInput?.focus();
        searchInput?.select();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [toggleSidebar]);

  const renderActiveScreen = () => {
    switch (activeScreen) {
      case 'overview':
        return <OverviewScreen />;
      case 'storage-explorer':
        return <StorageExplorerScreen />;
      case 'large-files':
        return <LargeFilesScreen />;
      case 'duplicates':
        return <DuplicateFinderScreen />;
      case 'cleanup-planner':
        return <CleanupPlannerScreen />;
      case 'safety-center':
        return <SafetyCenterScreen />;
      case 'developer-cleanup':
        return <DeveloperCleanupScreen />;
      case 'automations':
        return <AutomationsScreen />;
      case 'jarvis':
        return <JarvisScreen />;
      case 'activity-recovery':
        return <ActivityRecoveryScreen />;
      case 'settings':
        return <SettingsScreen />;
      default:
        return <OverviewScreen />;
    }
  };

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        width: '100vw',
        height: '100vh',
        backgroundColor: 'var(--bg-ground)',
        overflow: 'hidden',
      }}
    >
      {/* Top Application Toolbar */}
      <TopToolbar />

      {/* Main Workspace Frame */}
      <div
        style={{
          display: 'flex',
          flex: 1,
          width: '100%',
          overflow: 'hidden',
        }}
      >
        {/* Left Persistent Navigation Sidebar */}
        <Sidebar />

        {/* Central Workspace Canvas */}
        <main
          style={{
            flex: 1,
            height: '100%',
            overflow: 'hidden',
            backgroundColor: 'var(--bg-ground)',
            display: 'flex',
            flexDirection: 'column',
          }}
        >
          {renderActiveScreen()}
        </main>

        {/* Right Resizable Property Inspector */}
        <PropertyInspector />
      </div>

      {/* Bottom Global Status Bar */}
      <StatusBar />

      {/* Overlays & Dialogs */}
      <CommandPalette />
      <NotificationCenter />
      <ContextMenu />
      <Toast />
    </div>
  );
};
