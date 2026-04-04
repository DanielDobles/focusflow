import os

file_path = r"C:\Users\armon\DEV\HOI4_MD_FT\focusflow\src\app.rs"

with open(file_path, "r", encoding="utf-8") as f:
    content = f.read()

# 1. Close eframe::App and open FocusFlowApp
# We find exactly this transition:
# "        });\n    }\n\n    /// Left panel"
# And we change it to close the trait and open the struct impl.
bad_transition = """        });
    }

    /// Left panel"""

good_transition = """        });
    }
}

impl FocusFlowApp {
    /// Left panel"""
content = content.replace(bad_transition, good_transition)

# If it didn't find the exact match, try the python replacement differently
if bad_transition not in content:
    # Just insert it before "    fn ui_left_panel"
    old = "    fn ui_left_panel"
    new = "}\n\nimpl FocusFlowApp {\n    fn ui_left_panel"
    # only replace the first occurrence
    content = content.replace(old, new, 1)

# 2. Fix egui::Margin::symmetric floats to i8
content = content.replace("egui::Margin::symmetric(24.0, 16.0)", "egui::Margin::symmetric(24, 16)")
content = content.replace("egui::Margin::symmetric(0.0, 40.0)", "egui::Margin::symmetric(0, 40)")
content = content.replace("egui::Margin::symmetric(14.0, 40.0)", "egui::Margin::symmetric(14, 40)")

# 3. Remove .tracking(1.5)
content = content.replace(".tracking(1.5)", "")

# 4. Fix Frame::none() -> Frame::NONE
content = content.replace("Frame::none()", "Frame::NONE")

with open(file_path, "w", encoding="utf-8") as f:
    f.write(content)

print("Fixed rust compile errors")
