import type { PlayerState, Track } from '@/types';
import { TauriService } from '@/services/TauriService';

type Sub = (s: PlayerState) => void;

export class PlayerStore {
  private static _i: PlayerStore;
  static get instance() { return this._i ??= new PlayerStore(); }

  private s: PlayerState = {
    currentTrack: null, isPlaying: false, currentTime: 0, duration: 0, volume: 0.8, queue: [], queueIndex: -1
  };
  private subs = new Set<Sub>();
  private timer: number | null = null;
  private api = TauriService.instance;

  subscribe(fn: Sub){ this.subs.add(fn); fn(this.s); return () => this.subs.delete(fn); }
  private emit(){ for(const fn of this.subs) fn(this.s); }

  async playTrack(t: Track, queue?: Track[], index?: number){
    if (queue) { this.s.queue = queue; this.s.queueIndex = index ?? queue.indexOf(t); }
    this.s.currentTrack = t;
    await this.api.loadAudioFile(t.path);
    await this.api.play();
    this.s.isPlaying = true;
    this.startPolling();
    this.emit();
  }
  async toggle(){
    if (!this.s.currentTrack) return;
    if (this.s.isPlaying){ await this.api.pause(); this.s.isPlaying = false; this.emit(); }
    else { await this.api.play(); this.s.isPlaying = true; this.emit(); }
  }
  async stop(){
    await this.api.stop();
    this.s.isPlaying = false; this.s.currentTime = 0; this.emit();
    this.stopPolling();
  }
  async setVolume(v: number){
    this.s.volume = v; this.emit();
    await this.api.setVolume(v);
  }
  async seek(sec: number){
    this.s.currentTime = sec; this.emit();
    await this.api.seekTo(sec);
  }
  private startPolling(){
    if (this.timer) return;
    const tick = async () => {
      try {
        this.s.currentTime = await this.api.getPosition();
        this.s.duration = await this.api.getDuration();
        this.emit();
      } catch {}
    };
    // @ts-ignore
    this.timer = window.setInterval(tick, 500);
  }
  private stopPolling(){
    if (this.timer){ clearInterval(this.timer); this.timer = null; }
  }
  cleanup(){ this.stopPolling(); this.subs.clear(); }
}
