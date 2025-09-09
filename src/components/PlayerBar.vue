<<template>
  <div class="player-card player-bottom">
    <div class="player-card__inner">
      <!-- Left: Now Playing Info -->
      <div class="meta">
        <div class="artwork" aria-hidden="true">
          <img v-if="albumArt" :src="albumArt" :alt="title" />
          <span v-else>🎵</span>
        </div>
        <div class="info">
          <div class="title">
            {{ title || 'No track playing' }}
            <span v-if="artist" class="muted"> — {{ artist }}</span>
          </div>
          <div class="sub">
            <span>{{ fmt(currentTime) }}</span>
            <span class="dot">•</span>
            <span>{{ fmt(duration) }}</span>
            <span v-if="isBuffering" class="buffering">
              <span class="spinner" aria-hidden="true"></span>
              <span>buffering…</span>
            </span>
          </div>
        </div>
      </div>

      <!-- Center: Transport Controls & Progress -->
      <div class="controls-row">
        <div class="transport">
          <button
              class="icon-btn"
              title="Back 10s"
              @click="back10"
              :disabled="!canSeek"
          >
            ⏮
          </button>
          <button
              class="play-btn"
              :disabled="!canSeek"
              @click="togglePlay"
              :title="isPlaying ? 'Pause' : 'Play'"
          >
            {{ isPlaying ? '⏸' : '▶' }}
          </button>
          <button
              class="icon-btn"
              title="Forward 10s"
              @click="forward10"
              :disabled="!canSeek"
          >
            ⏭
          </button>
        </div>

        <!-- Progress bar below controls -->
        <div class="progress">
          <input
              type="range"
              min="0"
              max="100"
              :value="seekPct"
              :disabled="!canSeek"
              @input="onSeek"
              :style="`--progress: ${seekPct}%`"
          />
        </div>
      </div>

      <!-- Right: Volume -->
      <div class="volume">
        <button
            class="icon-btn"
            :title="muted ? 'Unmute' : 'Mute'"
            @click="toggleMute"
        >
          {{ muted ? '🔇' : '🔊' }}
        </button>
        <input
            type="range"
            min="0"
            max="100"
            :value="volPct"
            @input="onVolInput"
            :style="`--progress: ${volPct}%`"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { usePlayerStore } from '@/stores/player'

const p = usePlayerStore()
const { togglePlay, back10, forward10, seekToPercent, setVolumeLinear, toggleMute } = p

const title        = computed(() => p.title)
const artist       = computed(() => p.artist)
const isPlaying    = computed(() => p.isPlaying)
const isBuffering  = computed(() => p.isBuffering)
const currentTime  = computed(() => p.currentTime)
const duration     = computed(() => p.duration)
const canSeek      = computed(() => p.canSeek)
const muted        = computed(() => p.muted)
const volPct       = computed(() => Math.round(p.volume * 100))
const seekPct      = computed(() => p.duration ? (p.currentTime / p.duration) * 100 : 0)

function fmt(sec: number) {
  if (!isFinite(sec) || sec <= 0) return '0:00'
  const m = Math.floor(sec / 60)
  const s = Math.floor(sec % 60)
  return `${m}:${s.toString().padStart(2, '0')}`
}
function onSeek(e: Event) {
  const v = Number((e.target as HTMLInputElement).value || 0)
  seekToPercent(v)
}
function onVolInput(e: Event) {
  const v = Number((e.target as HTMLInputElement).value || 0)
  setVolumeLinear(v / 100)
}
</script>
