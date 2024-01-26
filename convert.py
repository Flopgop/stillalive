import mido
from mido import MidiFile, MidiTrack, MetaMessage

def midi_to_frequencies_and_durations(midi_file):
    midi = MidiFile(midi_file)
    ticks_per_beat = midi.ticks_per_beat
    tempo = 60000000 / 119  # Calculate microseconds per beat for 119 BPM

    frequencies_and_durations = []
    active_notes = {}  # To keep track of active notes and their start times

    for i, track in enumerate(midi.tracks):
        current_ticks = 0
        current_time = 0.0

        for msg in track:
            current_ticks += msg.time
            current_time = current_ticks / ticks_per_beat * (tempo / 1000000.0)

            if msg.type == 'note_on':
                active_notes[msg.note] = current_time

            elif msg.type == 'note_off':
                if msg.note in active_notes:
                    start_time = active_notes[msg.note]
                    end_time = current_time
                    duration = end_time - start_time
                    frequency = 440 * (2 ** ((msg.note - 69) / 12))
                    frequencies_and_durations.append((start_time, frequency, duration))
                    del active_notes[msg.note]

            elif msg.type == 'set_tempo':
                tempo = msg.tempo

    return frequencies_and_durations

def write_result_to_file(result, output_file):
    with open(output_file, 'w') as file:
        file.write("""pub struct Note {
    pub frequency: f64,
    pub start_time: f64,
    pub duration: f64
}\n""")
        file.write(f"pub static NOTES: [Note; {len(result)}] = [\n")
        for start_time, frequency, duration in result:
            file.write(f"Note {{frequency: {frequency}, start_time: {start_time}, duration: {duration}}},\n")
        file.write("];")

# Replace 'your_midi_file.mid' with your actual MIDI file
midi_file_path = 'stillalive.mid'
output_file_path = 'frequencies.rs'

result = midi_to_frequencies_and_durations(midi_file_path)
print(result)

write_result_to_file(result, output_file_path)