import { Player } from './components/player/Player';
import { Library } from './components/library/Library';
import { Sidebar } from './components/navigation/Sidebar';
import { TauriService } from './services/TauriService';

export class App {
    private player: Player | null = null;
    private library: Library | null = null;
    private sidebar: Sidebar | null = null;
    private tauriService: TauriService;

    constructor() {
        this.tauriService = new TauriService();
    }

    async init(): Promise<void> {
        await this.setupTauri();
        this.initializeComponents();
        this.setupRouting();
    }

    private async setupTauri(): Promise<void> {
        await this.tauriService.init();
    }

    private initializeComponents(): void {
        const playerContainer = document.getElementById('player');
        const libraryContainer = document.getElementById('library');
        const sidebarContainer = document.getElementById('sidebar');

        if (playerContainer) {
            this.player = new Player(playerContainer);
        }

        if (libraryContainer) {
            this.library = new Library(libraryContainer);
        }

        if (sidebarContainer) {
            this.sidebar = new Sidebar(sidebarContainer);
        }
    }

    private setupRouting(): void {
        // Simple routing logic
    }
}