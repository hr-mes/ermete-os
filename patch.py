import json
import sys

file_path = "/home/ermete/.gemini/antigravity-cli/brain/6c56cba7-8eff-4414-a62f-43c968bed459/.system_generated/worktrees/subagent-Kernel-Event-Emitter-self-4880b2f8/system/ermete-ebpf-sched/src/ai_bridge.rs"
with open(file_path, "r") as f:
    content = f.read()

target1 = """                    if let Ok(classification) = serde_json::from_str::<AiProcessClassification>(&resp_str) {
                        return classification;
                    }"""
replacement1 = """                    if let Ok(classification) = serde_json::from_str::<AiProcessClassification>(&resp_str) {
                        self.notify_morphic_pill(&classification).await;
                        return classification;
                    }"""

target2 = """                            return AiProcessClassification {
                                pid,
                                binary_name: comm.to_string(),
                                recommended_sched_class: sched_class,
                                recommended_weight: weight as u32,
                                recommended_slice_us: slice,
                                heuristic_score: score,
                            };"""
replacement2 = """                            let classification = AiProcessClassification {
                                pid,
                                binary_name: comm.to_string(),
                                recommended_sched_class: sched_class,
                                recommended_weight: weight as u32,
                                recommended_slice_us: slice,
                                heuristic_score: score,
                            };
                            self.notify_morphic_pill(&classification).await;
                            return classification;"""

target3 = """        AiProcessClassification {
            pid,
            binary_name: comm.to_string(),
            recommended_sched_class: sched_class,
            recommended_weight: weight,
            recommended_slice_us: slice_us,
            heuristic_score,
        }
    }
}"""
replacement3 = """        let classification = AiProcessClassification {
            pid,
            binary_name: comm.to_string(),
            recommended_sched_class: sched_class,
            recommended_weight: weight,
            recommended_slice_us: slice_us,
            heuristic_score,
        };

        self.notify_morphic_pill(&classification).await;

        classification
    }

    async fn notify_morphic_pill(&self, class: &AiProcessClassification) {
        if matches!(class.recommended_sched_class, SchedClass::InteractiveUi | SchedClass::BatchCompute) {
            if let Some(conn) = &self.connection {
                let payload = serde_json::json!({
                    "activity_type": "AiSchedulingEvent",
                    "process_name": class.binary_name,
                    "pid": class.pid,
                    "sched_class": format!("{:?}", class.recommended_sched_class),
                    "priority_score": class.heuristic_score,
                    "message": format!("Neural inference classified {} as {:?}", class.binary_name, class.recommended_sched_class),
                }).to_string();

                let _ = conn.call_method(
                    Some("os.ermete.Shell"),
                    "/os/ermete/Shell/LiveActivity",
                    Some("os.ermete.Shell.LiveActivity"),
                    "UpdateActivity",
                    &(payload.as_str()),
                ).await;
            }
        }
    }
}"""

if target1 not in content or target2 not in content or target3 not in content:
    print("Error: Target not found.")
    if target1 not in content: print("target1 not found")
    if target2 not in content: print("target2 not found")
    if target3 not in content: print("target3 not found")
    sys.exit(1)

content = content.replace(target1, replacement1)
content = content.replace(target2, replacement2)
content = content.replace(target3, replacement3)

with open(file_path, "w") as f:
    f.write(content)

print("Patch applied successfully.")
