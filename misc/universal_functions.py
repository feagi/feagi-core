
import json
import datetime
import os.path
import glob
import pickle
from datetime import datetime
from genethesizer import calculate_fitness
import IPU_vision
import db_handler


global parameters
if 'parameters' not in globals():
    with open("./configuration/parameters.json", "r") as data_file:
        parameters = json.load(data_file)
        # print("Parameters has been read from file")


# live_mode_status can hvae modes of idle, learning, testing, tbd
live_mode_status = 'idle'
regenerate = True

training_neuron_list_utf = []
training_neuron_list_img = []
labeled_image = []

global brain_is_running
brain_is_running = False
brain_run_id = ""


if parameters["Switches"]["capture_brain_activities"]:
    global fcl_history
    fcl_history = {}


class InjectorParams:
    img_flag = False
    utf_flag = False
    injection_has_begun = False
    variation_handler = True
    exposure_handler = True
    utf_handler = True
    variation_counter = parameters["Auto_injector"]["variation_default"]
    exposure_counter = parameters["Auto_injector"]["exposure_default"]
    utf_counter = parameters["Auto_injector"]["utf_default"]
    variation_counter_actual = variation_counter
    exposure_counter_actual = exposure_counter
    utf_counter_actual = utf_counter
    injection_start_time = datetime.now()
    num_to_inject = ''
    utf_to_inject = ''
    injection_mode = ''


class TesterParams:
    img_flag = False
    utf_flag = False
    testing_has_begun = False
    variation_handler = True
    exposure_handler = True
    utf_handler = True
    variation_counter = parameters["Auto_tester"]["variation_default"]
    exposure_counter = parameters["Auto_tester"]["exposure_default"]
    utf_counter = parameters["Auto_tester"]["utf_default"]
    variation_counter_actual = variation_counter
    exposure_counter_actual = exposure_counter
    utf_counter_actual = utf_counter
    test_start_time = datetime.now()
    num_to_inject = ''
    test_mode = ''
    comprehension_counter = 0
    test_attempt_counter = 0
    # temp_stats = []
    test_stats = {}
    test_id = ""

    # Load copy of all MNIST training images into mnist_data in form of an iterator. Each object has image label + image


mnist_iterator = IPU_vision.read_mnist_raw()
mnist_array = []
for _ in mnist_iterator:
    mnist_array.append(_)
# print(len(mnist_array))


# Reads the list of all Cortical areas defined in Genome
def cortical_list():
    # print("%%% Genome: ", genome)
    blueprint = genome["blueprint"]
    cortical_list = []
    for key in blueprint:
        cortical_list.append(key)
    return cortical_list


def cortical_group_members(group):
    members = []
    for item in cortical_list():
        if genome['blueprint'][item]['group_id'] == group:
            members.append(item)
    return members


def cortical_sub_group_members(group):
    members = []
    for item in cortical_list():
        if genome['blueprint'][item]['sub_group_id'] == group:
            members.append(item)
    return members


def load_genome_metadata_in_memory():
    with open(parameters["InitData"]["genome_file"], "r") as data_file:
        genome_db = json.load(data_file)
        genome_metadata = genome_db["genome_metadata"]
    return genome_metadata


global genome_metadata
genome_metadata = load_genome_metadata_in_memory()


def load_genome_in_memory():
    from genethesizer import select_a_genome
    genome = select_a_genome()
    # print("NNN", type(genome), genome)
    # global genome_id
    # genome_id = genome["genome_id"]
    return genome


# Resets the in-memory brain for each cortical area
def reset_brain():
    cortical_areas = cortical_list()
    for item in cortical_areas:
        brain[item] = {}
    return brain


def load_rules_in_memory():
    with open(parameters["InitData"]["rules_path"], "r") as data_file:
        rules = json.load(data_file)
    # print("Rules has been successfully loaded into memory...")
    return rules


def load_brain_in_memory():
    cortical_areas = cortical_list()
    brain = {}
    for item in cortical_areas:
        if os.path.isfile(parameters["InitData"]["connectome_path"] + item + '.json'):
            with open(parameters["InitData"]["connectome_path"] + item + '.json', "r") as data_file:
                data = json.load(data_file)
                brain[item] = data
    # print("Brain has been successfully loaded into memory...")
    return brain


def save_brain_to_disk(cortical_area='all'):
    global brain
    if cortical_area != 'all':
        with open(parameters["InitData"]["connectome_path"]+cortical_area+'.json', "r+") as data_file:
            data = brain[cortical_area]
            # print("...All data related to Cortical area %s is saved in connectome\n" % cortical_area)
            # Saving changes to the connectome
            data_file.seek(0)  # rewind
            data_file.write(json.dumps(data, indent=3))
            data_file.truncate()
    else:
        for cortical_area in cortical_list():
            with open(parameters["InitData"]["connectome_path"]+cortical_area+'.json', "r+") as data_file:
                data = brain[cortical_area]
                print(">>> >>> All data related to Cortical area %s is saved in connectome" % cortical_area)
                # Saving changes to the connectome
                data_file.seek(0)  # rewind
                data_file.write(json.dumps(data, indent=3))
                data_file.truncate()
    return


def save_genome_to_disk():
    mongo = db_handler.MongoManagement()
    global genome_test_stats, genome, genome_id

    genome_db = {}
    genome_db["genome_id"] = genome_id
    genome_db["generation_date"] = str(datetime.now())
    genome_db["properties"] = genome

    brain_fitness = calculate_fitness(genome_test_stats)
    genome_db["fitness"] = brain_fitness

    print("Brain fitness factor was evaluated as: ", brain_fitness)

    mongo.insert_genome(genome_db)

    for stat in genome_test_stats:
        stat_to_save = stat
        mongo.insert_test_stats(stat_to_save)

    print("Genome has been preserved for future generations!")

    return


def reset_cumulative_counter_instances():
    """
    To reset the cumulative counter instances
    """
    for cortical_area in brain:
        for neuron in brain[cortical_area]:
            brain[cortical_area][neuron]['cumulative_fire_count_inst'] = 0
    return


def toggle_verbose_mode():
    if parameters["Switches"]["verbose"]:
        parameters["Switches"]["verbose"] = False
        print("Verbose mode is Turned OFF!")
    else:
        parameters["Switches"]["verbose"] = True
        print("Verbose mode is Turned On!")


def toggle_injection_mode():
    if parameters["Auto_injector"]["injector_status"]:
        parameters["Auto_injector"]["injector_status"] = False
        print("Auto_train mode is Turned OFF!")
    else:
        parameters["Auto_injector"]["injector_status"] = True
        print("Auto_train mode is Turned On!")


def toggle_test_mode():
    if parameters["Auto_tester"]["tester_status"]:
        parameters["Auto_tester"]["tester_status"] = False
        print("Auto_test mode is Turned OFF!")
    else:
        parameters["Auto_tester"]["tester_status"] = True
        print("Auto_test mode is Turned On!")


def toggle_brain_status():
    global brain_is_running
    if brain_is_running:
        brain_is_running = False
        print("Brain is not running!")
    else:
        brain_is_running = True
        print("Brain is now running!!!")


def save_fcl_to_disk():
    global fcl_history
    global brain_run_id
    with open("./fcl_repo/fcl-" + brain_run_id + ".json", 'w') as fcl_file:
        # Saving changes to the connectome
        fcl_file.seek(0)  # rewind
        fcl_file.write(json.dumps(fcl_history, indent=3))
        fcl_file.truncate()

    print("Brain activities has been preserved!")


def load_fcl_in_memory(file_name):
    with open(file_name, 'r') as fcl_file:
        fcl_data = json.load(fcl_file)
    return fcl_data


def latest_fcl_file():
    list_of_files = glob.glob('./fcl_repo/*.json')  # * means all if need specific format then *.csv
    latest_file = max(list_of_files, key=os.path.getctime)
    return latest_file


def pickler(data, id):
    id = brain_run_id
    with open("./pickle_jar/fcl-" + id + ".pkl", 'wb') as output:
        pickle.dump(data, output)


def unpickler(data_type, id):
    if data_type == 'fcl':
        with open("./pickle_jar/fcl-" + id + ".pkl", 'rb') as input_data:
            data = pickle.load(input_data)
    else:
        print("Error: Type not found!")
    return data


global genome_id
genome_id = ""

global genome
genome = load_genome_in_memory()

global genome_stats
genome_stats = {}

global genome_test_stats
genome_test_stats = []


global blueprint
blueprint = cortical_list()

global brain
brain = load_brain_in_memory()

global event_id
event_id = '_'

global cortical_areas
cortical_areas = cortical_list()

global rules
rules = load_rules_in_memory()
