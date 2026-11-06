"""What has changed upstream in the Coopt sources kayak's files were taken
from (C368): each says so, `coopt-source: PATH@REV`, as Fixpt's ports say
`fixpt-source:` (scripts/fixpt-drift.py, whose `main` this runs), against
the pin COOPT_REV and the clone ../kayak-refs/coopt ($KAYAK_COOPT_CLONE).
Moving the pin: fetch the clone from its GitHub origin, check out the new
commit, write it into COOPT_REV, carry over or decide against each change
this lists, and move each mark.

usage: python3 -I scripts/coopt-drift.py [--check]
"""

import importlib.util
import os

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("fixpt_drift", os.path.join(HERE, "fixpt-drift.py"))
drift = importlib.util.module_from_spec(spec)
spec.loader.exec_module(drift)

if __name__ == "__main__":
    drift.main("coopt")
