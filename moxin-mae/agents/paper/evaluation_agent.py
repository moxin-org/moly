#!/usr/bin/envs python3
# -*- coding: utf-8 -*-
import json
from dora import DoraStatus
import pyarrow as pa

from mae.kernel.utils.log import write_agent_log
from mae.kernel.utils.util import load_agent_config
from mae.run.run import run_dspy_agent, run_crewai_agent



class Operator:
    def on_event(
            self,
            dora_event,
            send_output,
    ) -> DoraStatus:
        if dora_event["type"] == "INPUT":

            if dora_event['id'] == 'refinement_result':
                inputs = load_agent_config('./use_case/evaluation_agent.yml')
                result = """
                """
                dora_result = json.loads(dora_event["value"][0].as_py())
                config_values = json.loads(dora_event["value"][1].as_py())

                # Use provided API key that comes from Moxin
                inputs['model_api_key'] = config_values["model_api_key"]

                inputs['context'] = dora_result.get('context')
                local_iterations = dora_result.get('local_iterations', None)
                if local_iterations is not None:
                    max_iterations = inputs.get('max_iterations')
                    if local_iterations > max_iterations:
                        print('Task Result:  ',dora_result.get('context'))
                        log_result = {"7, " + inputs.get('log_step_name', "Step_one"): dora_result.get('context')}

                        write_agent_log(log_type=inputs.get('log_type', None),
                                        log_file_path=inputs.get('log_path', None),
                                        data=log_result)
                    else:
                        print('inputs    :  ',inputs)
                        print("dora_result   :", dora_result)
                        if 'agents' not in inputs.keys():
                            inputs['task'] = dora_result['task']

                            result = run_dspy_agent(inputs=inputs)
                        else:
                            result = run_crewai_agent(crewai_config=inputs)
                        log_result = {"7, " + inputs.get('log_step_name', "Step_one"): result}

                        if 'Yes' in result or 'yes' in result or '是' in result:
                            print(f"In the {dora_result.get('local_iterations')} iteration result  : ", inputs['context'])
                            write_agent_log(log_type=inputs.get('log_type', None),
                                            log_file_path=inputs.get('log_path', None),
                                            data=log_result)
                        else:
                            inputs['local_iterations'] = local_iterations + 1
                            if inputs['local_iterations'] <= max_iterations :
                                result = { 'context': dora_result.get('context'),'local_iterations':inputs['local_iterations'],'rag_data':dora_result['rag_data'],'task':dora_result['task']}

                                # Carry on Moxin config values from previous step
                                send_output("evaluation_result", pa.array([json.dumps(result), json.dumps(config_values)]),dora_event['metadata'])
                    # send_output("feedback_result", pa.array([json.dumps(result)]),dora_event['metadata'])  # add this line
        return DoraStatus.CONTINUE


