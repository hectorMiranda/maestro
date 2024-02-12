from flask import Flask, jsonify
import random
from datetime import datetime

app = Flask(__name__)

def generar_nombre_aleatorio():
    nombres = ["Juan", "Ana", "Pedro", "Luisa"]
    apellidos = ["Pérez", "Rodríguez", "Gómez", "López"]
    return f"{random.choice(nombres)} {random.choice(apellidos)}"

def generar_fecha_nacimiento_aleatoria():
    start_dt = datetime.strptime('1970-01-01', '%Y-%m-%d')
    end_dt = datetime.strptime('2000-12-31', '%Y-%m-%d')
    random_dt = start_dt + (end_dt -
