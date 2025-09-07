import type { Track } from '@/types';
import { TauriService } from '@/services/TauriService';

type Sub = (items: Track[]) => void;

export class LibraryStore {
  private static _i: LibraryStore;
  static get instance(){ return this._i ??= new LibraryStore(); }

  private api = TauriService.instance;
  private tracks: Track[] = [];
  private subs = new Set<Sub>();

  subscribe(fn: Sub){ this.subs.add(fn); fn(this.tracks); return () => this.subs.delete(fn); }
  private emit(){ for(const fn of this.subs) fn(this.tracks); }

  async refresh(){
    this.tracks = await this.api.listTracks();
    this.emit();
  }
  async scan(){
    await this.api.scanLibrary();
    await this.refresh();
  }
  getAll(){ return this.tracks; }
}
