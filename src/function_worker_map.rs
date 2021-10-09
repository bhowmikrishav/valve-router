use std::collections::BTreeMap;
use std::vec::Vec;

type StaticName = [u8; 64];

fn static_name_from_str(s: &str) -> StaticName {
    let mut name = [0u8; 64];
    for (i, c) in s.bytes().enumerate() {
        name[i] = c;
    }
    name
}

struct FunctionWorkerConfig {
    function_name: StaticName, // Must be unique for a functionDefinition.
    worker_uuid: StaticName,   // Must be unique for a worker.
    timeout: u32,              // Hard-limit timeout in ms for the function to complete.
    traffic: u32,              // Ongoing functions exections on the Worker
    max_concurrency: u32,      // Maximum number of concurrent function execution on the worker.
}

impl FunctionWorkerConfig {
    fn copy(&self) -> FunctionWorkerConfig {
        FunctionWorkerConfig {
            function_name: self.function_name.clone(),
            worker_uuid: self.worker_uuid.clone(),
            timeout: self.timeout,
            traffic: self.traffic,
            max_concurrency: self.max_concurrency,
        }
    }
}

// FunctionWorkerConfig examples
// {
//     "function_name": "function_name1",
//     "worker_uuid": "worker_uuid1",
//     "timeout": 10000,
//     "traffic": 0,
//     "max_concurrency": 1
// }
// {
//    "function_name": "function_name2",
//   "worker_uuid": "worker_uuid2",
//  "timeout": 10000,
// "traffic": 0,
// "max_concurrency": 1
// }
// {
//     "function_name": "function_name1",
//     "worker_uuid": "worker_uuid3",
//     "timeout": 10000,
//     "traffic": 0,
//     "max_concurrency": 1
// }

// # Function Worker Map
// - Maps function names to their available workers confings
// - FunctionName is the Key
// - FunctionWorkerConfig Array is the Value

struct FunctionWorkerMap {
    map: BTreeMap<StaticName, Vec<FunctionWorkerConfig>>,
}

struct LeastBusyWorkerError {
    error_code: u32,
    error_message: String,
}

impl FunctionWorkerMap {
    pub fn new() -> FunctionWorkerMap {
        FunctionWorkerMap {
            map: BTreeMap::new(),
        }
    }

    pub fn insert_worker_config(
        &mut self,
        function_name: StaticName,
        worker_config: FunctionWorkerConfig,
    ) {
        let worker_config_vec_option = self.map.get_mut(&function_name);
        // Check if the worker config Array already exists
        if worker_config_vec_option.is_none() {
            let mut worker_config_vec: Vec<FunctionWorkerConfig> = Vec::new();
            worker_config_vec.push(worker_config);
            self.map.insert(function_name, worker_config_vec);
        } else {
            let worker_config_vec = worker_config_vec_option.unwrap();
            worker_config_vec.push(worker_config);

            // Decend Sort the worker configs based on the traffic `max_concurrency - traffic`
            worker_config_vec.sort_by(|a, b| {
                let a_traffic = a.traffic;
                let b_traffic = b.traffic;
                let a_max_concurrency = a.max_concurrency;
                let b_max_concurrency = b.max_concurrency;
                let a_traffic_ratio = (a_max_concurrency - a_traffic) as i32;
                let b_traffic_ratio = (b_max_concurrency - b_traffic) as i32;
                b_traffic_ratio.cmp(&a_traffic_ratio)
            });
        }
    }

    pub fn get_least_busy_worker(
        &mut self,
        function_name: StaticName,
    ) -> Result<FunctionWorkerConfig, LeastBusyWorkerError> {
        let worker_config_vec_option = self.map.get(&function_name);
        // check if any worker is mapped for the function
        if worker_config_vec_option.is_none() {
            return Result::Err(LeastBusyWorkerError {
                error_code: 1,
                error_message: "No worker is mapped for the function".to_string(),
            });
        }

        let worker_config_vec = worker_config_vec_option.unwrap();

        if worker_config_vec.len() == 0 {
            return Result::Err(LeastBusyWorkerError {
                error_code: 2,
                error_message: "No worker is mapped for the function".to_string(),
            });
        }

        // return the least busy worker
        let least_busy_worker = worker_config_vec[0].copy();
        Ok(least_busy_worker)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    /// - Create a FunctionWorkerMap
    /// - Insert 1st worker config example, using insert_worker_config
    /// - Assert that the worker config Array is not empty
    /// - Verify that the worker config Array contains the 1st worker config example
    fn test_insert_worker_config() {
        let mut function_worker_map = FunctionWorkerMap::new();

        let function_name: StaticName = static_name_from_str("test-function1");
        let worker_uuid: StaticName = static_name_from_str("test-worker1");
        let timeout: u32 = 10000;
        let traffic: u32 = 0;
        let max_concurrency: u32 = 1;

        let worker_config: FunctionWorkerConfig = FunctionWorkerConfig {
            function_name: function_name,
            worker_uuid: worker_uuid,
            timeout: timeout,
            traffic: traffic,
            max_concurrency: max_concurrency,
        };

        function_worker_map.insert_worker_config(function_name, worker_config);
        let worker_config_vec_option = function_worker_map.map.get(&function_name);
        assert_eq!(worker_config_vec_option.is_some(), true);
        let worker_config_vec = worker_config_vec_option.unwrap();
        assert_eq!(worker_config_vec.len(), 1);
        assert_eq!(worker_config_vec[0].function_name, function_name);
        assert_eq!(worker_config_vec[0].worker_uuid, worker_uuid);
        assert_eq!(worker_config_vec[0].timeout, timeout);
        assert_eq!(worker_config_vec[0].traffic, traffic);
        assert_eq!(worker_config_vec[0].max_concurrency, max_concurrency);
    }

    #[test]
    /// - Create a FunctionWorkerMap
    /// - Insert 1st worker config (func1) example, using insert_worker_config
    /// - Insert 2nd worker config (func2) example, using insert_worker_config
    /// - Insert 3rd worker config (func1) example, using insert_worker_config
    /// - Assert that the worker config Array for func1 has 2 items
    /// - Assert that the worker config Array for func2 has 1 item
    /// - Verify values of each worker
    fn test_insert_multiple_worker_config() {
        let mut function_worker_map = FunctionWorkerMap::new();

        let function_name1: StaticName = static_name_from_str("test-function1");
        let worker_uuid1: StaticName = static_name_from_str("test-worker1");
        let timeout1: u32 = 10000;
        let traffic1: u32 = 0;
        let max_concurrency1: u32 = 1;

        let worker_config: FunctionWorkerConfig = FunctionWorkerConfig {
            function_name: function_name1,
            worker_uuid: worker_uuid1,
            timeout: timeout1,
            traffic: traffic1,
            max_concurrency: max_concurrency1,
        };

        function_worker_map.insert_worker_config(function_name1, worker_config);

        let function_name: StaticName = static_name_from_str("test-function2");
        let worker_uuid: StaticName = static_name_from_str("test-worker2");
        let timeout: u32 = 10000;
        let traffic: u32 = 0;
        let max_concurrency: u32 = 1;

        let worker_config: FunctionWorkerConfig = FunctionWorkerConfig {
            function_name: function_name,
            worker_uuid: worker_uuid,
            timeout: timeout,
            traffic: traffic,
            max_concurrency: max_concurrency,
        };

        function_worker_map.insert_worker_config(function_name, worker_config);

        let function_name3: StaticName = static_name_from_str("test-function1");
        let worker_uuid3: StaticName = static_name_from_str("test-worker3");
        let timeout3: u32 = 10000;
        let traffic3: u32 = 0;
        let max_concurrency3: u32 = 2;

        let worker_config: FunctionWorkerConfig = FunctionWorkerConfig {
            function_name: function_name3,
            worker_uuid: worker_uuid3,
            timeout: timeout3,
            traffic: traffic3,
            max_concurrency: max_concurrency3,
        };

        function_worker_map.insert_worker_config(function_name3, worker_config);

        let worker_config_vec_option = function_worker_map.map.get(&function_name3);

        assert_eq!(worker_config_vec_option.is_some(), true);
        let worker_config_vec = worker_config_vec_option.unwrap();
        assert_eq!(worker_config_vec.len(), 2);
        assert_eq!(worker_config_vec[0].function_name, function_name3);
        assert_eq!(worker_config_vec[0].worker_uuid, worker_uuid3);
        // assert_eq!(worker_config_vec[0].timeout, timeout);
        // assert_eq!(worker_config_vec[0].traffic, traffic);
        
        assert_eq!(worker_config_vec[1].function_name, function_name1);
        assert_eq!(worker_config_vec[1].worker_uuid, worker_uuid1);
        // assert_eq!(worker_config_vec[1].timeout, timeout1);
    }
}
