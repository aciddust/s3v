export interface ContextMenuItem {
  label: string;
  action: () => void;
  disabled?: boolean;
  separator?: boolean;
}

export interface ContextMenuState {
  open: boolean;
  x: number;
  y: number;
  items: ContextMenuItem[];
}

export type PanelSide = 'left' | 'right';

class UIStore {
  dualPanel = $state<boolean>(false);
  activePanel = $state<PanelSide>('left');
  sidebarWidth = $state<number>(220);
  transferPanelVisible = $state<boolean>(false);
  transferPanelExpanded = $state<boolean>(true);
  searchQuery = $state<string>('');
  profileManagerOpen = $state<boolean>(false);
  rightPanelProfileId = $state<string | null>(null);

  contextMenu = $state<ContextMenuState>({
    open: false,
    x: 0,
    y: 0,
    items: [],
  });

  toggleDualPanel(): void {
    this.dualPanel = !this.dualPanel;
    if (!this.dualPanel) {
      this.activePanel = 'left';
      this.rightPanelProfileId = null;
    }
  }

  setActivePanel(side: PanelSide): void {
    this.activePanel = side;
  }

  setRightPanelProfile(profileId: string | null): void {
    this.rightPanelProfileId = profileId;
  }

  toggleTransferPanel(): void {
    this.transferPanelVisible = !this.transferPanelVisible;
  }

  showTransferPanel(): void {
    this.transferPanelVisible = true;
    this.transferPanelExpanded = true;
  }

  toggleTransferPanelContent(): void {
    this.transferPanelExpanded = !this.transferPanelExpanded;
  }

  openContextMenu(x: number, y: number, items: ContextMenuItem[]): void {
    this.contextMenu = { open: true, x, y, items };
  }

  closeContextMenu(): void {
    this.contextMenu = { ...this.contextMenu, open: false, items: [] };
  }
}

export const uiStore = new UIStore();
