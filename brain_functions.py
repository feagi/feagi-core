

import multiprocessing as mp
import universal_functions
import sys
from tty import setraw
import termios


class Brain:
    @staticmethod
    def print_basic_info():
        import stats
        cp = mp.current_process()
        print("\rstarting", cp.name, cp.pid)
        print("\rConnectome database =                  %s" %
              universal_functions.parameters["InitData"]["connectome_path"])
        print("\rTotal neuron count in the connectome  %s  is: %i" %
              (universal_functions.parameters["InitData"]["connectome_path"] + 'vision_v1.json',
               stats.connectome_neuron_count(cortical_area='vision_v1')))
        print(' \rexiting', cp.name, cp.pid)
        return

    @staticmethod
    def show_cortical_areas():
        import visualizer
        # The following visualizes the connectome. Pass neighbor_show='true' as a parameter to view neuron relationships
        # visualizer.connectome_visualizer(cortical_area='vision_v1', neighbor_show='true')
        # visualizer.connectome_visualizer(cortical_area='vision_v2', neighbor_show='true')
        # visualizer.connectome_visualizer(cortical_area='vision_IT', neighbor_show='true')
        visualizer.connectome_visualizer(cortical_area='vision_memory', neighbor_show='true')
        return

    def see_from_mnist(self, mnist_img, fcl_queue, event_queue):
        cp = mp.current_process()
        # print(' starting', cp.name, cp.pid)
#        settings.reset_cumulative_counter_instances()   # ????
        fire_list= self.read_from_mnist(mnist_img, event_queue)
        self.inject_to_fcl(fire_list, fcl_queue)
        # print(' exiting', cp.name, cp.pid)
        return

    @staticmethod
    def read_from_mnist(mnist_img, event_queue):
        # print("Reading from MNIST")
        import architect
        import IPU_vision
        import universal_functions
        import mnist
        # Read image from MNIST database and translate them to activation in vision_v1 neurons & injects to FCL
        init_fire_list = []
#        IPU_vision_array = IPU_vision.convert_image_to_coordinates(mnist.read_image(image_number)[0])    # todo  ?????
        cortical_list = universal_functions.cortical_list()
        vision_group = []
        for item in cortical_list:
            if universal_functions.genome['blueprint'][item]['sub_group_id'] == 'vision_v1':
                vision_group.append(item)
        # print('vision group is: ', vision_group)
        image_ = mnist_img
        # image_ = mnist.read_image(image_number)
        image = image_[0]
        print("*** Image read from MNIST was :", image_[1])
        # print('image :\n ', np.array2string(image, max_line_width=np.inf))
        filter = IPU_vision.Filter()
        filtered_image = filter.brightness(image)
        # print('Filtered image :\n ', np.array2string(filter.brightness(image), max_line_width=np.inf))
        for cortical_area in vision_group:
            cortical_direction_sensitivity = universal_functions.genome['blueprint'][cortical_area]['direction_sensitivity']
            kernel_size = 7
            polarized_image = IPU_vision.create_direction_matrix(filtered_image, kernel_size, cortical_direction_sensitivity)
            # print("Polarized image for :", cortical_area)
            # print(np.array2string(np.array(polarized_image), max_line_width=np.inf))
            ipu_vision_array = IPU_vision.convert_direction_matrix_to_coordinates(polarized_image)
            neuron_id_list = IPU_vision.convert_image_locations_to_neuron_ids(ipu_vision_array, cortical_area)
            for item in neuron_id_list:
                init_fire_list.append([cortical_area, item])
        # Event is an instance of time where an IPU event has occurred
        event_id = architect.event_id_gen()
        print(" <> <> <> <> <> <> <> <> An event related to mnist reading with following id has been logged:", event_id)
        event_queue.put(event_id)
        # print('Initial Fire List:')
        # print(init_fire_list)
        return init_fire_list

    @staticmethod
    def inject_to_fcl(fire_list, fcl_queue):
        # print("Injecting to FCL.../\/\/\/")
        # Update FCL with new input data. FCL is read from the Queue and updated
        flc = fcl_queue.get()
        for item in fire_list:
            flc.append(item)
        fcl_queue.put(flc)
        # print("Injected to FCL.../\/\/\/")
        return

    @staticmethod
    def read_char(char, fcl_queue):
        import IPU_utf8
        cp = mp.current_process()
        # print(' starting', cp.name, cp.pid)
        if char:
            fire_list = IPU_utf8.convert_char_to_fire_list(char)
            print(fire_list)
            print("% % % % % % % % % % % % % % % % % % Injecting character >>>  %s <<<" % char)
            Brain.inject_to_fcl(fire_list, fcl_queue)
        # print(' exiting', cp.name, cp.pid)

    @staticmethod
    def read_user_input():
        print("Func: read_user_input : start")
        fd = sys.stdin.fileno()
        old_settings = termios.tcgetattr(fd)
        try:
            setraw(sys.stdin.fileno())
            universal_functions.parameters["Input"]["user_input"] = sys.stdin.read(1)
            # sys.stdout.write("\r%s" % user_input_queue)
            print("hahaha______hehehe")
        finally:
            termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)
            # sys.stdout.flush()
        print("Func: read_user_input : end")
        return

    # def kernel_view(self):
        # IPU_vision.image_read_block(mnist.read_image(image_number), 3, [27, 27])

        # print("Direction matrix with Kernel 3")
        # a = (IPU_vision.create_direction_matrix(mnist.read_image(image_number), 3))
        # for items in range(0, np.shape(a)[0]):
        #     print(' '.join(map(str, [x.encode('utf-8') for x in a[items]])))
        #
        # print("Direction matrix with Kernel 5")
        # a = (IPU_vision.create_direction_matrix(mnist.read_image(image_number), 5))
        # for items in range(0, np.shape(a)[0]):
        #     print(' '.join(map(str, [x.encode('utf-8') for x in a[items]])))
        #
        # print("Direction matrix with Kernel 7")
        # a = (IPU_vision.create_direction_matrix(mnist.read_image(image_number), 7))
        # for items in range(0, np.shape(a)[0]):
        #     print(' '.join(map(str, [x.encode('utf-8') for x in a[items]])))
        #
        #
        # print(IPU_vision.direction_stats(a))
        # return

    @staticmethod
    def cortical_area_neuron_count(cortical_area):
        """
        Returns number of Neurons in the connectome
        """
        data = universal_functions.brain[cortical_area]
        neuron_count = 0
        for key in data:
            neuron_count += 1
        return neuron_count

    @staticmethod
    def cortical_area_synapse_count(cortical_area):
        """
        Returns number of Neurons in the connectome
        """
        data = universal_functions.brain[cortical_area]
        synapse_count = 0
        for neuron in data:
            for _ in data[neuron]['neighbors']:
                synapse_count += 1
        return synapse_count

    def connectome_neuron_count(self):
        total_neuron_count = 0
        for cortical_area in universal_functions.cortical_areas:
            total_neuron_count += self.cortical_area_neuron_count(cortical_area)
        return total_neuron_count

    def connectome_synapse_count(self):
        total_synapse_count = 0
        for cortical_area in universal_functions.cortical_areas:
            total_synapse_count += self.cortical_area_synapse_count(cortical_area)

        return total_synapse_count