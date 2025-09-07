import type { Settings } from '@/types';
import { TauriService } from '@/services/TauriService';

type Sub = (s: Settings) => void;

export class SettingsStore {
  private static _i: SettingsStore;
  static get instance(){ return this._i ??= new SettingsStore(); }

  private api = TauriService.instance;
  private s: Settings = { library_root: null, use_managed_dir: false, managed_root: null };
  private subs = new Set<Sub>();

  subscribe(fn: Sub){ this.subs.add(fn); fn(this.s); return () => this.subs.delete(fn); }
  private emit(){ for(const fn of this.subs) fn(this.s); }

  async load(){ this.s = await this.api.getSettings(); this.emit(); }
  async chooseAndSaveRoot(){
    const dir = await this.api.chooseLibraryDir();
    await this.api.setLibraryRoot(dir);
    await this.load();
  }
  get(){ return this.s; }
}
