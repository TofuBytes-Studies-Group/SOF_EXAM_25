import os
import sys

import requests
import re
this_dir = os.path.dirname(__file__)
python_root = os.path.abspath(os.path.join(this_dir))
if python_root not in sys.path:
    sys.path.append(python_root)


API_URL = "http://localhost:11434/api/generate"
MODEL = "hf.co/DavidAU/Gemma-The-Writer-Mighty-Sword-9B-GGUF:Q2_K"

def generate_weapon_name(base_name):
    prompt = f"""
        Create a Skyrim-style weapon name using the base '{base_name}'.
        The name should sound mystical or powerful, and follow formats like:
        - {base_name}'s Icefang Blade
        - {base_name}'s Vengeance
        - Blade of {base_name}

        Return only the name, no description.
    """
    headers = {"Content-Type": "application/json"}
    payload = {"model": "hf.co/DavidAU/Gemma-The-Writer-Mighty-Sword-9B-GGUF:Q2_K", "prompt": prompt, "stream": False}

    try:
        print("Generating weapon name...")
        response = requests.post(API_URL, json=payload, headers=headers)
        response.raise_for_status()
        name_text = response.json().get("response", "").strip()
        return name_text
    except Exception as e:
        print(f"Error generating weapon name: {e}")
        return f"{base_name}'s Weapon"

# Function to generate the full weapon stats
def generate_weapon(full_name):
    prompt = f"""
        Generate a Skyrim weapon named '{full_name}' with the following attributes:
        - Damage: A single integer value (e.g., 15).
        - Weight: A single integer value (e.g., 10).
        - Upgrade: The upgrade material (e.g., Diamond Ingot, Steel to Daedric).
        - Perk: A unique perk (e.g., Frostbite Cleave).
        - Type: The type of weapon (e.g., Sword, Axe, Bow).

        Do not use ranges for damage (e.g., 18-25), instead provide a single integer value. Format the output as:
        Damage: <value>, Weight: <value>, Upgrade: <value>, Perk: <value>, Type: <value>

        Ensure that Damage and Weight is an integer. 
    """
    headers = {"Content-Type": "application/json"}
    payload = {"model": "hf.co/DavidAU/Gemma-The-Writer-Mighty-Sword-9B-GGUF:Q2_K", "prompt": prompt, "stream": False}

    try:
        print("Sending request to AI for weapon stats...")
        response = requests.post(API_URL, json=payload, headers=headers)
        response.raise_for_status()
        generated_text = response.json().get("response", "")
        print("Generated text from AI:", generated_text)

        weapon_details = parse_generated_text(generated_text)
        weapon_details["Name"] = full_name
        return weapon_details
    except requests.exceptions.RequestException as e:
        print(f"Error connecting to AI API: {e}")
        return {}


def parse_generated_text(text):

    weapon_data = {}
    pattern = r"([A-Za-z]+):\s*([^,]+)"

    lines = text.strip().splitlines()
    for line in lines:
        matches = re.findall(pattern, line)
        for key, value in matches:
            if key == "Damage":
                weapon_data[key] = int(value)
            elif key == "Weight":
                weapon_data[key] = int(value)
            else:
                weapon_data[key] = value.strip()

    print("PARSED FIELDS:", weapon_data)

    required_fields = ["Damage", "Weight", "Upgrade", "Perk", "Type"]
    missing = [f for f in required_fields if f not in weapon_data]

    if missing:
        print(f"Not all required fields found. Missing: {missing}")
        return {}

    print("Parsed weapon:", weapon_data)
    return weapon_data