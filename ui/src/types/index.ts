export type Track = {
  path: string;
  title: string;
  artist: string;
  album?: string | null;
  duration_secs: number;
  has_art: boolean;
};

export type Settings = {
  library_root: string | null;
  use_managed_dir: boolean;
  managed_root: string | null;
};

export type PlayerState = {
  currentTrack: Track | null;
  isPlaying: boolean;
  currentTime: number; // seconds
  duration: number;    // seconds
  volume: number;      // 0..1
  queue: Track[];
  queueIndex: number;
};
