from joblib import load
import pandas as pd
import os

def predict_price(weapon_data: dict) -> float:
    """Predict weapon price from stats"""
    base_path = os.path.dirname(__file__)
    model = load(os.path.join(base_path, "models", "model.pkl"))
    encoder = load(os.path.join(base_path, "models", "ordinal_encoder.pkl"))

    for key in weapon_data:
        if isinstance(weapon_data[key], str):
            weapon_data[key] = weapon_data[key].strip().rstrip('.')


    # Creates DataFrame with EXACT SAME COLUMNS as training data
    df = pd.DataFrame([{
        'Damage': weapon_data['Damage'],
        'Weight': weapon_data['Weight'],
        'Upgrade': weapon_data['Upgrade'].rstrip('.') if isinstance(weapon_data['Upgrade'], str) else weapon_data['Upgrade'],
        'Perk': weapon_data['Perk'].rstrip('.') if isinstance(weapon_data['Perk'], str) else weapon_data['Perk'],
        'Type': weapon_data['Type'].rstrip('.') if isinstance(weapon_data['Type'], str) else weapon_data['Type']
    }])

    # This ensures the DataFrame has the same columns as the model
    df = df.reindex(columns=model.feature_names_in_, fill_value=0)

    if 'ordinal_encoder.pkl' in globals():
        df_encoded = encoder.transform(df)
        return float(model.predict(df_encoded)[0])
    else:
        return float(model.predict(df)[0])